## Purpose
TBD

## Requirements
### Requirement: Circuit breaker error counting
The system SHALL track errors per (group_id, server_id) in Redis using key `cb:err:{group_id}:{server_id}`. Each error SHALL increment the counter via INCR. When the key is first created, its TTL SHALL be set to the server's `cb_window_seconds`. The counter auto-resets when the TTL expires.

#### Scenario: First error increments counter and sets TTL
- **WHEN** a server with `cb_max_failures=5, cb_window_seconds=60` encounters its first error
- **THEN** the system SHALL set `cb:err:{group_id}:{server_id}` to 1 with TTL=60 seconds

#### Scenario: Subsequent error increments existing counter
- **WHEN** `cb:err:{group_id}:{server_id}` exists with value 3 and another error occurs
- **THEN** the system SHALL increment the counter to 4 without changing the TTL

#### Scenario: Counter resets after window expires
- **WHEN** `cb:err:{group_id}:{server_id}` TTL expires (60 seconds pass with fewer than max_failures errors)
- **THEN** the key SHALL be deleted by Redis and the next error starts a fresh count

### Requirement: Circuit breaker trips when threshold reached
The system SHALL trip the circuit breaker when the error count reaches `cb_max_failures`. Tripping SHALL set Redis key `cb:open:{group_id}:{server_id}` with value `1` and TTL = `cb_cooldown_seconds`. The error counter key SHALL be deleted after tripping.

#### Scenario: Error count reaches threshold — circuit trips
- **WHEN** `cb:err:{group_id}:{server_id}` reaches 5 (equal to `cb_max_failures=5`)
- **THEN** the system SHALL SET `cb:open:{group_id}:{server_id}` with TTL=`cb_cooldown_seconds`, DELETE the error counter key, and the server SHALL be skipped in subsequent requests

#### Scenario: Circuit already open — no duplicate trip
- **WHEN** a server errors and `cb:open:{group_id}:{server_id}` already exists
- **THEN** the system SHALL still increment the error counter but SHALL NOT re-SET the open key (idempotent)

### Requirement: Circuit breaker auto re-enable
The system SHALL automatically re-enable a circuit-broken server when the `cb:open:{group_id}:{server_id}` key expires (TTL reaches 0). No explicit action is needed — Redis key expiry handles re-enable.

#### Scenario: Cooldown expires — server available again
- **WHEN** `cb:open:{group_id}:{server_id}` TTL expires after `cb_cooldown_seconds`
- **THEN** the key SHALL no longer exist and the server SHALL be included in the failover loop on the next request

### Requirement: Circuit breaker not configured — no-op
When a group-server assignment has `cb_max_failures` as NULL, the circuit breaker logic SHALL be completely skipped for that server. No Redis keys SHALL be read or written.

#### Scenario: Server without circuit breaker configured
- **WHEN** a server has `cb_max_failures=NULL` and encounters errors
- **THEN** the system SHALL NOT create any `cb:err` or `cb:open` Redis keys and SHALL NOT skip the server due to circuit breaker logic

### Requirement: Circuit status API
The system SHALL provide `GET /api/admin/groups/{group_id}/circuit-status` that returns the circuit breaker state for all servers in the group.

#### Scenario: One server circuit-open
- **WHEN** server S1 has `cb:open:{group_id}:{S1_id}` with 150 seconds remaining TTL and server S2 has no open key
- **THEN** the endpoint SHALL return `[{"server_id": "S1_id", "is_open": true, "remaining_seconds": 150}, {"server_id": "S2_id", "is_open": false, "remaining_seconds": 0}]`

#### Scenario: No servers have circuit breaker configured
- **WHEN** no servers in the group have `cb_max_failures` configured
- **THEN** the endpoint SHALL return an empty array `[]`
