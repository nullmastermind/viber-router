## Purpose
TBD

## Requirements
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

### Requirement: Group-server assignment has max_input_tokens field
The `group_servers` table SHALL include a nullable integer column `max_input_tokens` that defaults to NULL. New assignments SHALL be created with `max_input_tokens=NULL` (no threshold). The field SHALL be included in `GroupServerDetail` and `AdminGroupServerDetail` responses.

#### Scenario: New assignment defaults to no token threshold
- **WHEN** an admin assigns a server to a group without specifying `max_input_tokens`
- **THEN** the assignment SHALL have `max_input_tokens=NULL`

#### Scenario: AdminGroupServerDetail includes max_input_tokens
- **WHEN** the admin fetches a group detail via GET `/api/admin/groups/{group_id}`
- **THEN** each server in the `servers` array SHALL include `max_input_tokens` (nullable integer)

### Requirement: Update max_input_tokens via assignment endpoint
The system SHALL allow setting and clearing `max_input_tokens` via PUT `/api/admin/groups/{group_id}/servers/{server_id}`. The value MUST be either NULL (no limit) or a positive integer >= 1.

#### Scenario: Set max_input_tokens to a positive value
- **WHEN** admin sends PUT with `{"max_input_tokens": 30000}`
- **THEN** the system SHALL update the field, invalidate the group's Redis cache, and return the updated assignment

#### Scenario: Clear max_input_tokens by setting to null
- **WHEN** admin sends PUT with `{"max_input_tokens": null}`
- **THEN** the system SHALL set `max_input_tokens=NULL`, invalidate the group's Redis cache, and return the updated assignment

#### Scenario: Omit max_input_tokens — no change
- **WHEN** admin sends PUT without a `max_input_tokens` field
- **THEN** the system SHALL leave the existing `max_input_tokens` value unchanged

### Requirement: Group-server assignment has supported_models field
The `group_servers` table SHALL include a `supported_models TEXT[] NOT NULL DEFAULT '{}'` column. New assignments SHALL be created with `supported_models = []` (empty array, no filtering). The field SHALL be included in `GroupServerDetail` and `AdminGroupServerDetail` responses.

#### Scenario: New assignment defaults to empty supported_models
- **WHEN** an admin assigns a server to a group without specifying `supported_models`
- **THEN** the assignment SHALL have `supported_models = []`

#### Scenario: AdminGroupServerDetail includes supported_models
- **WHEN** the admin fetches a group detail via GET `/api/admin/groups/{group_id}`
- **THEN** each server in the `servers` array SHALL include `supported_models` (array of strings, defaults to `[]`)

### Requirement: Update supported_models via assignment endpoint
The system SHALL allow setting and clearing `supported_models` via PUT `/api/admin/groups/{group_id}/servers/{server_id}`. The value MUST be an array of strings (empty array clears the filter). Omitting the field leaves the existing value unchanged.

#### Scenario: Set supported_models to a list of models
- **WHEN** admin sends PUT with `{"supported_models": ["gpt-4o", "gpt-4o-mini"]}`
- **THEN** the system SHALL update the field, invalidate the group's Redis cache, and return the updated assignment with `supported_models: ["gpt-4o", "gpt-4o-mini"]`

#### Scenario: Clear supported_models by setting to empty array
- **WHEN** admin sends PUT with `{"supported_models": []}`
- **THEN** the system SHALL set `supported_models = []`, invalidate the group's Redis cache, and return the updated assignment

#### Scenario: Omit supported_models — no change
- **WHEN** admin sends PUT without a `supported_models` field
- **THEN** the system SHALL leave the existing `supported_models` value unchanged

### Requirement: Group-server assignment has active hours fields
The `group_servers` table SHALL include three nullable TEXT columns: `active_hours_start`, `active_hours_end`, and `active_hours_timezone`. All three default to NULL. A NULL set means the server is active 24/7. The fields SHALL be included in `GroupServerDetail` and `AdminGroupServerDetail` responses. All three fields SHALL carry `#[serde(default)]` in Rust models to ensure backward compatibility when deserializing cached Redis data that predates this change.

#### Scenario: New assignment defaults to no active hours restriction
- **WHEN** an admin assigns a server to a group without specifying active hours fields
- **THEN** the assignment SHALL have `active_hours_start=NULL`, `active_hours_end=NULL`, and `active_hours_timezone=NULL`

#### Scenario: AdminGroupServerDetail includes active hours fields
- **WHEN** the admin fetches a group detail via GET `/api/admin/groups/{group_id}`
- **THEN** each server in the `servers` array SHALL include `active_hours_start`, `active_hours_end`, and `active_hours_timezone` (all nullable strings)

#### Scenario: Existing cached GroupConfig deserializes with new fields absent
- **WHEN** a `GroupConfig` object cached in Redis before this deployment is deserialized
- **THEN** the three active hours fields SHALL default to `None` (i.e., 24/7 behavior) without a deserialization error

### Requirement: Update active hours configuration via assignment endpoint
The system SHALL allow setting active hours fields via PUT `/api/admin/groups/{group_id}/servers/{server_id}`. All three fields MUST be either all provided (non-null) or all null (all-or-nothing). The `active_hours_start` and `active_hours_end` values MUST match the pattern `HH:MM` where HH is 00-23 and MM is 00-59. The `active_hours_timezone` value MUST be a valid IANA timezone string recognized by the `chrono-tz` crate. Omitting all three fields leaves the existing values unchanged. On successful update, the system SHALL call `invalidate_group_all_keys()` to clear the Redis cache.

#### Scenario: Set active hours — all three fields provided and valid
- **WHEN** admin sends PUT with `{"active_hours_start": "08:00", "active_hours_end": "23:00", "active_hours_timezone": "Asia/Ho_Chi_Minh"}`
- **THEN** the system SHALL update all three fields, invalidate the group's Redis cache, and return the updated assignment

#### Scenario: Clear active hours — all three null
- **WHEN** admin sends PUT with `{"active_hours_start": null, "active_hours_end": null, "active_hours_timezone": null}`
- **THEN** the system SHALL set all three fields to NULL, invalidate the group's Redis cache, and return the updated assignment

#### Scenario: Omit active hours fields — no change
- **WHEN** admin sends PUT without any active hours fields
- **THEN** the system SHALL leave the existing active hours values unchanged

#### Scenario: Partial active hours config — validation error
- **WHEN** admin sends PUT with `{"active_hours_start": "08:00", "active_hours_end": "23:00"}` (missing timezone)
- **THEN** the system SHALL return HTTP 400 with error message explaining all-or-nothing requirement

#### Scenario: Invalid time format — validation error
- **WHEN** admin sends PUT with `{"active_hours_start": "8:00", "active_hours_end": "23:00", "active_hours_timezone": "UTC"}`
- **THEN** the system SHALL return HTTP 400 with error message indicating the time format must be HH:MM

#### Scenario: Invalid timezone string — validation error
- **WHEN** admin sends PUT with `{"active_hours_start": "08:00", "active_hours_end": "23:00", "active_hours_timezone": "Not/ATimezone"}`
- **THEN** the system SHALL return HTTP 400 with error message indicating the timezone is not a recognized IANA timezone

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
