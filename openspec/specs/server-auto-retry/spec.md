## ADDED Requirements

### Requirement: Per-server retry configuration stored in group_servers
The `group_servers` table SHALL include three nullable columns: `retry_status_codes INTEGER[]`, `retry_count INTEGER`, and `retry_delay_seconds DOUBLE PRECISION`. All three default to NULL. A NULL set means retry is disabled for that server. New assignments SHALL be created with all three as NULL.

#### Scenario: New assignment defaults to no retry config
- **WHEN** an admin assigns a server to a group without specifying retry fields
- **THEN** the assignment SHALL have `retry_status_codes=NULL`, `retry_count=NULL`, and `retry_delay_seconds=NULL`

#### Scenario: GroupServerDetail includes retry fields
- **WHEN** the admin fetches a group detail via GET `/api/admin/groups/{group_id}`
- **THEN** each server in the `servers` array SHALL include `retry_status_codes` (nullable integer array), `retry_count` (nullable integer), and `retry_delay_seconds` (nullable float)

#### Scenario: Existing cached GroupConfig deserializes with retry fields absent
- **WHEN** a `GroupConfig` object cached in Redis before this deployment is deserialized
- **THEN** the three retry fields SHALL default to `None` (retry disabled) without a deserialization error

### Requirement: Retry config validation is all-or-nothing
The system SHALL enforce that all three retry fields are either all non-null (retry enabled) or all null (retry disabled). Partial configuration SHALL be rejected. When enabled: `retry_count` MUST be >= 1; `retry_delay_seconds` MUST be > 0; each value in `retry_status_codes` MUST be an integer in the range 400–599 inclusive; `retry_status_codes` MUST be non-empty.

#### Scenario: Valid retry config — all three fields provided
- **WHEN** admin sends PUT with `{"retry_status_codes": [503, 429], "retry_count": 2, "retry_delay_seconds": 1.0}`
- **THEN** the system SHALL accept the config, update the assignment, invalidate the group's Redis cache, and return the updated assignment

#### Scenario: Clear retry config — all three null
- **WHEN** admin sends PUT with `{"retry_status_codes": null, "retry_count": null, "retry_delay_seconds": null}`
- **THEN** the system SHALL set all three fields to NULL, invalidate the group's Redis cache, and return the updated assignment

#### Scenario: Partial retry config — validation error
- **WHEN** admin sends PUT with `{"retry_status_codes": [503], "retry_count": 2}` (missing `retry_delay_seconds`)
- **THEN** the system SHALL return HTTP 400 with error message explaining all-or-nothing requirement

#### Scenario: retry_count less than 1 — validation error
- **WHEN** admin sends PUT with `{"retry_status_codes": [503], "retry_count": 0, "retry_delay_seconds": 1.0}`
- **THEN** the system SHALL return HTTP 400 with error message indicating `retry_count` must be >= 1

#### Scenario: retry_delay_seconds zero or negative — validation error
- **WHEN** admin sends PUT with `{"retry_status_codes": [503], "retry_count": 1, "retry_delay_seconds": 0.0}`
- **THEN** the system SHALL return HTTP 400 with error message indicating `retry_delay_seconds` must be > 0

#### Scenario: Status code out of range — validation error
- **WHEN** admin sends PUT with `{"retry_status_codes": [200, 503], "retry_count": 1, "retry_delay_seconds": 1.0}`
- **THEN** the system SHALL return HTTP 400 with error message indicating status codes must be in range 400–599

#### Scenario: Empty retry_status_codes array — validation error
- **WHEN** admin sends PUT with `{"retry_status_codes": [], "retry_count": 1, "retry_delay_seconds": 1.0}`
- **THEN** the system SHALL return HTTP 400 with error message indicating `retry_status_codes` must be non-empty

### Requirement: Retry config UI on server row in group detail page
The admin UI SHALL display a retry config icon button (`replay`) on each server row in the group detail page. Clicking the button SHALL open a dialog to configure the three retry fields. When retry is configured (all three non-null), the server row SHALL display a badge or indicator showing retry is active.

#### Scenario: Retry button opens config dialog
- **WHEN** admin clicks the `replay` icon button on a server row
- **THEN** a dialog SHALL open pre-populated with the server's current retry config (or empty defaults if not configured)

#### Scenario: Save retry config from dialog
- **WHEN** admin fills in valid retry fields and clicks Save
- **THEN** the system SHALL call PUT `/api/admin/groups/{group_id}/servers/{server_id}` with the retry fields and update the UI on success

#### Scenario: Clear retry config from dialog
- **WHEN** admin clears all retry fields and clicks Save
- **THEN** the system SHALL send all three fields as null and the server row badge SHALL be removed

#### Scenario: Badge shown when retry is configured
- **WHEN** a server has all three retry fields set (non-null)
- **THEN** the server row SHALL display a visual indicator (badge or chip) showing retry is active
