## ADDED Requirements

### Requirement: Group-server assignment has rate limit fields
The `group_servers` table SHALL include two nullable integer columns: `max_requests` and `rate_window_seconds`. Both default to NULL. New assignments SHALL be created with both as NULL (rate limiting disabled).

#### Scenario: New assignment defaults to no rate limit
- **WHEN** an admin assigns a server to a group without specifying rate limit fields
- **THEN** the assignment SHALL have `max_requests=NULL` and `rate_window_seconds=NULL`

#### Scenario: GroupServerDetail includes rate limit fields
- **WHEN** the admin fetches a group detail via GET `/api/admin/groups/{group_id}`
- **THEN** each server in the `servers` array SHALL include `max_requests` and `rate_window_seconds` (nullable integers)

### Requirement: Update rate limit configuration via assignment endpoint
The system SHALL allow setting rate limit fields via PUT `/api/admin/groups/{group_id}/servers/{server_id}`. Both fields MUST be either both null or both non-null (all-or-nothing validation). Each non-null value MUST be >= 1.

#### Scenario: Set rate limit config — both provided
- **WHEN** admin sends PUT with `{"max_requests": 100, "rate_window_seconds": 60}`
- **THEN** the system SHALL update both fields, invalidate the group's Redis cache, and return the updated assignment

#### Scenario: Partial rate limit config — validation error
- **WHEN** admin sends PUT with `{"max_requests": 100}` (missing `rate_window_seconds`)
- **THEN** the system SHALL return HTTP 400 with error message explaining all-or-nothing requirement

#### Scenario: Clear rate limit config — both null
- **WHEN** admin sends PUT with `{"max_requests": null, "rate_window_seconds": null}`
- **THEN** the system SHALL set both to NULL and invalidate the group's Redis cache

#### Scenario: Invalid values — validation error
- **WHEN** admin sends PUT with `{"max_requests": 0, "rate_window_seconds": 60}`
- **THEN** the system SHALL return HTTP 400 with error message (values must be >= 1)

#### Scenario: Mixed null and non-null — validation error
- **WHEN** admin sends PUT with `{"max_requests": 100, "rate_window_seconds": null}`
- **THEN** the system SHALL return HTTP 400 with error message explaining all-or-nothing requirement
