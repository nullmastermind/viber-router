## Purpose
TBD

## Requirements
### Requirement: Record unique user-agents per group
On each proxy request, after group config resolution, the system SHALL extract the `User-Agent` request header. If the header is absent or empty, the system SHALL normalize it to the string `"(empty)"`. The system SHALL then call `add_group_ua(redis, group_id, user_agent)` which runs `SADD group:{group_id}:user_agents {ua}` in Redis. If the SADD returns 1 (new member), the system SHALL insert a row into `group_user_agents(group_id, user_agent, first_seen_at)` using `ON CONFLICT DO NOTHING`. This entire operation SHALL be performed in a `tokio::spawn` task (fire-and-forget) and SHALL NOT block the proxy request.

#### Scenario: New user-agent recorded
- **WHEN** a proxy request arrives with `User-Agent: MyClient/1.0` for a group that has never seen this UA
- **THEN** the system SHALL SADD the UA to `group:{group_id}:user_agents`, receive return value 1, and insert a row into `group_user_agents` with `first_seen_at = now()`

#### Scenario: Known user-agent skipped
- **WHEN** a proxy request arrives with `User-Agent: MyClient/1.0` for a group that has already seen this UA (SADD returns 0)
- **THEN** the system SHALL NOT insert into `group_user_agents` (no DB write)

#### Scenario: Missing user-agent normalized
- **WHEN** a proxy request arrives with no `User-Agent` header
- **THEN** the system SHALL treat the UA as `"(empty)"` for both Redis SADD and DB insert

#### Scenario: Empty user-agent normalized
- **WHEN** a proxy request arrives with `User-Agent:` (empty value)
- **THEN** the system SHALL treat the UA as `"(empty)"` for both Redis SADD and DB insert

#### Scenario: Recording does not block request
- **WHEN** a proxy request arrives and UA recording is triggered
- **THEN** the recording SHALL run in a background task and the proxy response SHALL not be delayed by it

#### Scenario: Redis failure on SADD — fail open
- **WHEN** Redis is unavailable when `add_group_ua` is called
- **THEN** the system SHALL skip the DB insert and log the error, without affecting the proxy response

#### Scenario: DB failure on insert — fire-and-forget
- **WHEN** the DB insert into `group_user_agents` fails
- **THEN** the system SHALL log the error and the proxy response SHALL not be affected

#### Scenario: Concurrent SADD for same UA — race handled
- **WHEN** two concurrent requests for the same group arrive with the same UA simultaneously
- **THEN** Redis SADD atomicity ensures only one returns 1; the DB insert uses `ON CONFLICT DO NOTHING` to handle any remaining race

### Requirement: group_user_agents database table
The system SHALL maintain a `group_user_agents` table with columns: `group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE`, `user_agent TEXT NOT NULL`, `first_seen_at TIMESTAMPTZ NOT NULL DEFAULT now()`. The PRIMARY KEY SHALL be `(group_id, user_agent)`.

#### Scenario: Cascade delete on group removal
- **WHEN** a group is deleted
- **THEN** all rows in `group_user_agents` for that group SHALL be automatically deleted via ON DELETE CASCADE

#### Scenario: Duplicate insert ignored
- **WHEN** an insert is attempted for a `(group_id, user_agent)` pair that already exists
- **THEN** the insert SHALL be silently ignored (`ON CONFLICT DO NOTHING`)

### Requirement: add_group_ua cache function
The system SHALL expose a function `add_group_ua(redis: &Pool, group_id: Uuid, user_agent: &str) -> Result<bool>` in `cache.rs`. The function SHALL run `SADD group:{group_id}:user_agents {user_agent}` and return `true` if the return value is 1 (new member), `false` otherwise.

#### Scenario: New member returns true
- **WHEN** `add_group_ua` is called with a UA not yet in the set
- **THEN** the function SHALL return `Ok(true)`

#### Scenario: Existing member returns false
- **WHEN** `add_group_ua` is called with a UA already in the set
- **THEN** the function SHALL return `Ok(false)`
