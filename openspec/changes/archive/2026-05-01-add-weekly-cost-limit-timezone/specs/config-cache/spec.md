## ADDED Requirements

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
