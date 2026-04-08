## ADDED Requirements

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
