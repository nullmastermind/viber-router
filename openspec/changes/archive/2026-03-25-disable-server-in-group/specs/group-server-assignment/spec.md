## ADDED Requirements

### Requirement: Group-server assignment has is_enabled field
The `group_servers` table SHALL include an `is_enabled` boolean column that defaults to `true`. New server assignments SHALL be created with `is_enabled = true`.

#### Scenario: New server assignment defaults to enabled
- **WHEN** an authenticated admin sends POST `/api/admin/groups/{group_id}/servers` with `{"server_id": "uuid", "priority": 1}`
- **THEN** the system SHALL create the assignment with `is_enabled: true`

#### Scenario: GroupServerDetail includes is_enabled
- **WHEN** the admin fetches a group detail via GET `/api/admin/groups/{group_id}`
- **THEN** each server in the `servers` array SHALL include `is_enabled: true` or `is_enabled: false`

### Requirement: Toggle server enabled status within a group
The system SHALL allow toggling `is_enabled` via the existing update assignment endpoint.

#### Scenario: Disable a server in a group
- **WHEN** an authenticated admin sends PUT `/api/admin/groups/{group_id}/servers/{server_id}` with `{"is_enabled": false}`
- **THEN** the system SHALL set `is_enabled = false` on the assignment, invalidate the group's Redis cache, and return the updated assignment

#### Scenario: Re-enable a server in a group
- **WHEN** an authenticated admin sends PUT `/api/admin/groups/{group_id}/servers/{server_id}` with `{"is_enabled": true}`
- **THEN** the system SHALL set `is_enabled = true` on the assignment, invalidate the group's Redis cache, and return the updated assignment

#### Scenario: Toggle does not affect other fields
- **WHEN** an authenticated admin sends PUT with only `{"is_enabled": false}`
- **THEN** the system SHALL only change `is_enabled`; `priority` and `model_mappings` SHALL remain unchanged
