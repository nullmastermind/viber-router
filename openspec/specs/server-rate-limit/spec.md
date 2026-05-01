## Purpose
TBD

## Requirements
### Requirement: Rate limiter checks request count against configured limit
The rate limiter module SHALL check the current request count for a group-server pair in Redis using key `rl:{group_id}:{server_id}`. If the count is >= `max_requests`, the server SHALL be considered rate-limited. If Redis is unavailable, the check SHALL return false (fail open — server is not rate-limited).

#### Scenario: Server under limit
- **WHEN** the rate limiter checks a server with `max_requests=100` and the Redis counter shows 50
- **THEN** the check SHALL return false (not rate-limited)

#### Scenario: Server at limit
- **WHEN** the rate limiter checks a server with `max_requests=100` and the Redis counter shows 100
- **THEN** the check SHALL return true (rate-limited)

#### Scenario: Server over limit
- **WHEN** the rate limiter checks a server with `max_requests=100` and the Redis counter shows 150
- **THEN** the check SHALL return true (rate-limited)

#### Scenario: No counter exists (first request in window)
- **WHEN** the rate limiter checks a server and the Redis key `rl:{group_id}:{server_id}` does not exist
- **THEN** the check SHALL return false (not rate-limited, count is 0)

#### Scenario: Redis unavailable during check
- **WHEN** the rate limiter attempts to check the counter but Redis connection fails
- **THEN** the check SHALL return false (fail open)

### Requirement: Rate limiter increments counter with TTL window
The rate limiter module SHALL increment the Redis counter for a group-server pair using INCR on key `rl:{group_id}:{server_id}`. If the key is new (count becomes 1 after INCR), the module SHALL set TTL to `rate_window_seconds`. If Redis is unavailable, the increment SHALL be silently skipped.

#### Scenario: First request in window — set TTL
- **WHEN** the rate limiter increments the counter and the resulting count is 1
- **THEN** the module SHALL set EXPIRE on the key to `rate_window_seconds`

#### Scenario: Subsequent request in window — no TTL change
- **WHEN** the rate limiter increments the counter and the resulting count is > 1
- **THEN** the module SHALL NOT modify the TTL (existing expiry continues)

#### Scenario: Window expires — counter resets
- **WHEN** the TTL on `rl:{group_id}:{server_id}` expires
- **THEN** the key SHALL be deleted by Redis automatically, and the next request starts a new window

#### Scenario: Redis unavailable during increment
- **WHEN** the rate limiter attempts to increment but Redis connection fails
- **THEN** the increment SHALL be silently skipped (no error propagated)
