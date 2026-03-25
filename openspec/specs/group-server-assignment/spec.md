## MODIFIED Requirements

### Requirement: Assign a server to a group
The system SHALL allow assigning a server to a group with a priority number and optional model_mappings. The Redis cache for the group SHALL be invalidated. The server's short_id SHALL be included in the group-server detail response.

#### Scenario: Successful assignment
- **WHEN** an authenticated admin sends POST `/api/admin/groups/{group_id}/servers` with `{"server_id": "uuid", "priority": 1, "model_mappings": {"claude-opus-4-6": "my-opus"}}`
- **THEN** the system SHALL create the group_servers entry, invalidate the group's Redis cache, and the server detail SHALL include the server's short_id

#### Scenario: Duplicate server assignment
- **WHEN** an authenticated admin assigns a server that is already assigned to the group
- **THEN** the system SHALL return HTTP 409 with an error message

## ADDED Requirements

### Requirement: GroupServerDetail includes short_id and optional api_key
The GroupServerDetail response object SHALL include the server's `short_id` (integer) field. The `api_key` field SHALL be optional (nullable) to reflect that servers may not have a default key.

#### Scenario: Server with default key in group detail
- **WHEN** a group contains a server that has a default api_key and short_id 1
- **THEN** the GroupServerDetail SHALL include `short_id: 1` and `api_key: "sk-..."`

#### Scenario: Server without default key in group detail
- **WHEN** a group contains a server that has no default api_key and short_id 2
- **THEN** the GroupServerDetail SHALL include `short_id: 2` and `api_key: null`

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
