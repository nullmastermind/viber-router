## ADDED Requirements

### Requirement: Update retry configuration via assignment endpoint
The system SHALL allow setting and clearing retry fields (`retry_status_codes`, `retry_count`, `retry_delay_seconds`) via PUT `/api/admin/groups/{group_id}/servers/{server_id}`. All three fields MUST be either all non-null (retry enabled) or all null (retry disabled). Omitting all three fields leaves the existing retry config unchanged. On successful update, the system SHALL invalidate the group's Redis cache.

#### Scenario: Set retry config — all three fields provided and valid
- **WHEN** admin sends PUT with `{"retry_status_codes": [503, 429], "retry_count": 2, "retry_delay_seconds": 1.5}`
- **THEN** the system SHALL update all three fields, invalidate the group's Redis cache, and return the updated assignment

#### Scenario: Clear retry config — all three null
- **WHEN** admin sends PUT with `{"retry_status_codes": null, "retry_count": null, "retry_delay_seconds": null}`
- **THEN** the system SHALL set all three fields to NULL, invalidate the group's Redis cache, and return the updated assignment

#### Scenario: Omit retry fields — no change
- **WHEN** admin sends PUT without any retry fields
- **THEN** the system SHALL leave the existing retry config unchanged

#### Scenario: Partial retry config — validation error
- **WHEN** admin sends PUT with only some retry fields set (e.g., `retry_count` provided but `retry_status_codes` omitted)
- **THEN** the system SHALL return HTTP 400 with error message explaining all-or-nothing requirement
