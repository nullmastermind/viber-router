## ADDED Requirements

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
