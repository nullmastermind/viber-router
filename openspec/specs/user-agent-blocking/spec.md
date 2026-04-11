## ADDED Requirements

### Requirement: Block user-agents per group in proxy
The proxy SHALL check the incoming request's User-Agent (normalized: absent or empty becomes `"(empty)"`) against `GroupConfig.blocked_user_agents`. This check SHALL occur after the `is_active` check and before the `servers.is_empty()` check. If the UA matches any entry in `blocked_user_agents` (exact string match), the proxy SHALL return HTTP 403 with `error_type: "permission_error"` and `message: "Access denied"` using the path-appropriate error format.

#### Scenario: Blocked user-agent returns 403
- **WHEN** a proxy request arrives with `User-Agent: BadBot/2.0` and the group's `blocked_user_agents` contains `"BadBot/2.0"`
- **THEN** the proxy SHALL return HTTP 403 with `{"type":"error","error":{"type":"permission_error","message":"Access denied"}}` (Anthropic format for `/v1/messages`)

#### Scenario: Blocked empty user-agent returns 403
- **WHEN** a proxy request arrives with no `User-Agent` header and the group's `blocked_user_agents` contains `"(empty)"`
- **THEN** the proxy SHALL return HTTP 403

#### Scenario: Non-blocked user-agent passes through
- **WHEN** a proxy request arrives with `User-Agent: GoodClient/1.0` and `"GoodClient/1.0"` is NOT in `blocked_user_agents`
- **THEN** the proxy SHALL proceed normally to the failover waterfall

#### Scenario: Empty blocked list — no blocking
- **WHEN** a group has an empty `blocked_user_agents` list
- **THEN** the proxy SHALL not block any request based on UA

#### Scenario: Block check after is_active, before servers check
- **WHEN** a group is active but has a blocked UA and no servers configured
- **THEN** the proxy SHALL return 403 (UA block takes precedence over empty servers check)

### Requirement: group_blocked_user_agents database table
The system SHALL maintain a `group_blocked_user_agents` table with columns: `group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE`, `user_agent TEXT NOT NULL`, `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`. The PRIMARY KEY SHALL be `(group_id, user_agent)`. An index SHALL exist on `group_blocked_user_agents(group_id)`.

#### Scenario: Cascade delete on group removal
- **WHEN** a group is deleted
- **THEN** all rows in `group_blocked_user_agents` for that group SHALL be automatically deleted via ON DELETE CASCADE

### Requirement: blocked_user_agents in GroupConfig
`GroupConfig` SHALL include a field `blocked_user_agents: Vec<String>` annotated with `#[serde(default)]`. This field SHALL be populated during `resolve_group_config` by querying `group_blocked_user_agents` for the group, after the `key_allowed_models` query and before building the GroupConfig struct. The field SHALL be serialized into the cached GroupConfig in Redis.

#### Scenario: Blocked list loaded into config
- **WHEN** `resolve_group_config` is called for a group with two blocked UAs
- **THEN** `GroupConfig.blocked_user_agents` SHALL contain both UA strings

#### Scenario: Empty blocked list — default
- **WHEN** `resolve_group_config` is called for a group with no blocked UAs
- **THEN** `GroupConfig.blocked_user_agents` SHALL be an empty Vec (via `#[serde(default)]`)

#### Scenario: Old cached config deserializes cleanly
- **WHEN** a cached GroupConfig in Redis was serialized before `blocked_user_agents` was added
- **THEN** deserialization SHALL succeed with `blocked_user_agents` defaulting to empty Vec

### Requirement: User-agent blocking admin API
The system SHALL expose the following endpoints nested under `GET|POST|DELETE /api/admin/groups/{group_id}/user-agents`:

- `GET /api/admin/groups/{group_id}/user-agents` — returns all recorded UAs for the group from `group_user_agents`, ordered by `first_seen_at DESC`
- `GET /api/admin/groups/{group_id}/user-agents/blocked` — returns all blocked UAs from `group_blocked_user_agents`, ordered by `created_at DESC`
- `POST /api/admin/groups/{group_id}/user-agents/blocked` — adds a UA to the blocked list. Body: `{"user_agent": "string"}`. Calls `invalidate_group_all_keys`. Returns HTTP 201.
- `DELETE /api/admin/groups/{group_id}/user-agents/blocked` — removes a UA from the blocked list. Body: `{"user_agent": "string"}`. Calls `invalidate_group_all_keys`. Returns HTTP 204.

#### Scenario: List recorded user-agents
- **WHEN** an admin sends GET `/api/admin/groups/{id}/user-agents`
- **THEN** the system SHALL return a JSON array of `{user_agent, first_seen_at}` objects ordered by `first_seen_at DESC`

#### Scenario: List blocked user-agents
- **WHEN** an admin sends GET `/api/admin/groups/{id}/user-agents/blocked`
- **THEN** the system SHALL return a JSON array of `{user_agent, created_at}` objects ordered by `created_at DESC`

#### Scenario: Block a user-agent
- **WHEN** an admin sends POST `/api/admin/groups/{id}/user-agents/blocked` with `{"user_agent": "BadBot/2.0"}`
- **THEN** the system SHALL insert into `group_blocked_user_agents`, call `invalidate_group_all_keys`, and return HTTP 201

#### Scenario: Unblock a user-agent
- **WHEN** an admin sends DELETE `/api/admin/groups/{id}/user-agents/blocked` with `{"user_agent": "BadBot/2.0"}`
- **THEN** the system SHALL delete from `group_blocked_user_agents`, call `invalidate_group_all_keys`, and return HTTP 204

#### Scenario: Block UA with special characters
- **WHEN** an admin sends DELETE `/api/admin/groups/{id}/user-agents/blocked` with `{"user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"}`
- **THEN** the system SHALL correctly identify and delete the UA using the request body (not a path param)
