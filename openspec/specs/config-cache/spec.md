## Purpose
TBD
## Requirements
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
The system SHALL invalidate relevant Redis cache entries immediately when admin operations modify group configuration or global settings.

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

#### Scenario: Settings blocked paths updated
- **WHEN** an admin updates settings via PUT `/api/admin/settings` and the request includes `blocked_paths`
- **THEN** the Redis key `settings:blocked_paths` SHALL be deleted so the next proxy request reloads from the database

### Requirement: Cache global timezone setting
The system SHALL cache the global timezone setting in Redis and use the cached value for runtime calendar week calculations. The cache value SHALL default to `Asia/Ho_Chi_Minh` when no database value exists.

#### Scenario: Timezone cache miss
- **WHEN** runtime code requests the configured timezone and `settings:timezone` is not present in Redis
- **THEN** the system SHALL load `settings.timezone` from PostgreSQL, default to `Asia/Ho_Chi_Minh` if absent, cache the value in Redis, and return it

#### Scenario: Timezone cache hit
- **WHEN** runtime code requests the configured timezone and `settings:timezone` exists in Redis
- **THEN** the system SHALL use the cached timezone without querying PostgreSQL

#### Scenario: Timezone invalidated on settings update
- **WHEN** an admin updates settings via PUT `/api/admin/settings` and the request includes `timezone`
- **THEN** the Redis key `settings:timezone` SHALL be deleted so the next runtime lookup reloads from PostgreSQL

