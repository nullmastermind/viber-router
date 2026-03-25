## ADDED Requirements

### Requirement: Cache group config in Redis on lookup
The system SHALL cache the full group configuration (group details, server list with priorities and model mappings, and optional count-tokens server detail) in Redis keyed by API key after the first database lookup. The server list SHALL only include servers where `is_enabled = true` in the `group_servers` table.

#### Scenario: Cache miss — load from DB
- **WHEN** a proxy request arrives with `x-api-key: sk-vibervn-xxx` and no Redis cache entry exists for this key
- **THEN** the system SHALL query PostgreSQL for the group config, including only servers where `group_servers.is_enabled = true`, store it in Redis as `group:config:sk-vibervn-xxx`, and use it for routing

#### Scenario: Cache hit
- **WHEN** a proxy request arrives with `x-api-key: sk-vibervn-xxx` and a Redis cache entry exists
- **THEN** the system SHALL use the cached config (including count-tokens server detail if present) without querying PostgreSQL

#### Scenario: Redis unavailable — fallback to DB
- **WHEN** Redis is unavailable (connection error)
- **THEN** the system SHALL fall back to querying PostgreSQL directly (with `is_enabled = true` filter) and continue serving the request

#### Scenario: Disabled server excluded from proxy
- **WHEN** a group has servers A (enabled), B (disabled), C (enabled) ordered by priority
- **THEN** the proxy failover chain SHALL contain only A and C; B SHALL not be attempted

#### Scenario: All servers disabled
- **WHEN** all servers in a group have `is_enabled = false`
- **THEN** the proxy SHALL return HTTP 429 with `"All upstream servers unavailable"`

### Requirement: Write-through cache invalidation
The system SHALL invalidate relevant Redis cache entries immediately when admin operations modify group configuration.

#### Scenario: Group updated
- **WHEN** a group's name, failover_status_codes, is_active, count_tokens_server_id, or count_tokens_model_mappings is updated
- **THEN** the Redis cache entry for that group's API key SHALL be deleted

#### Scenario: Group API key regenerated
- **WHEN** a group's API key is regenerated from `sk-vibervn-old` to `sk-vibervn-new`
- **THEN** the Redis cache entry for `sk-vibervn-old` SHALL be deleted

#### Scenario: Group-server assignment changed
- **WHEN** a server is added to, removed from, or reordered within a group
- **THEN** the Redis cache entry for that group's API key SHALL be deleted

#### Scenario: Server updated
- **WHEN** a server's base_url or api_key is updated
- **THEN** the Redis cache entries for ALL groups that reference this server (via `group_servers` OR via `count_tokens_server_id`) SHALL be deleted

#### Scenario: Group deleted
- **WHEN** a group is deleted
- **THEN** the Redis cache entry for that group's API key SHALL be deleted
