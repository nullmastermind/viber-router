## ADDED Requirements

### Requirement: Group-server assignment has circuit breaker fields
The `group_servers` table SHALL include three nullable integer columns: `cb_max_failures`, `cb_window_seconds`, `cb_cooldown_seconds`. All three default to NULL. New assignments SHALL be created with all three as NULL (circuit breaker disabled).

#### Scenario: New assignment defaults to no circuit breaker
- **WHEN** an admin assigns a server to a group without specifying circuit breaker fields
- **THEN** the assignment SHALL have `cb_max_failures=NULL`, `cb_window_seconds=NULL`, `cb_cooldown_seconds=NULL`

#### Scenario: GroupServerDetail includes circuit breaker fields
- **WHEN** the admin fetches a group detail via GET `/api/admin/groups/{group_id}`
- **THEN** each server in the `servers` array SHALL include `cb_max_failures`, `cb_window_seconds`, and `cb_cooldown_seconds` (nullable integers)

### Requirement: Update circuit breaker configuration via assignment endpoint
The system SHALL allow setting circuit breaker fields via PUT `/api/admin/groups/{group_id}/servers/{server_id}`. All three fields MUST be either all null or all non-null (all-or-nothing validation). Each non-null value MUST be >= 1.

#### Scenario: Set circuit breaker config — all three provided
- **WHEN** admin sends PUT with `{"cb_max_failures": 5, "cb_window_seconds": 60, "cb_cooldown_seconds": 300}`
- **THEN** the system SHALL update all three fields, invalidate the group's Redis cache, and return the updated assignment

#### Scenario: Partial circuit breaker config — validation error
- **WHEN** admin sends PUT with `{"cb_max_failures": 5, "cb_window_seconds": 60}` (missing `cb_cooldown_seconds`)
- **THEN** the system SHALL return HTTP 400 with error message explaining all-or-nothing requirement

#### Scenario: Clear circuit breaker config — all null
- **WHEN** admin sends PUT with `{"cb_max_failures": null, "cb_window_seconds": null, "cb_cooldown_seconds": null}`
- **THEN** the system SHALL set all three to NULL and invalidate the group's Redis cache

#### Scenario: Invalid values — validation error
- **WHEN** admin sends PUT with `{"cb_max_failures": 0, "cb_window_seconds": 60, "cb_cooldown_seconds": 300}`
- **THEN** the system SHALL return HTTP 400 with error message (values must be >= 1)
