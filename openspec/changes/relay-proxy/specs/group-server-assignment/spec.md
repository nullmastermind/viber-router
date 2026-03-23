## ADDED Requirements

### Requirement: Assign a server to a group
The system SHALL allow assigning a server to a group with a priority number and optional model_mappings. The Redis cache for the group SHALL be invalidated.

#### Scenario: Successful assignment
- **WHEN** an authenticated admin sends POST `/api/admin/groups/{group_id}/servers` with `{"server_id": "uuid", "priority": 1, "model_mappings": {"claude-opus-4-6": "my-opus"}}`
- **THEN** the system SHALL create the group_servers entry and invalidate the group's Redis cache

#### Scenario: Duplicate server assignment
- **WHEN** an authenticated admin assigns a server that is already assigned to the group
- **THEN** the system SHALL return HTTP 409 with an error message

### Requirement: Update a group-server assignment
The system SHALL allow updating the priority and model_mappings of an existing group-server assignment.

#### Scenario: Update priority
- **WHEN** an authenticated admin sends PUT `/api/admin/groups/{group_id}/servers/{server_id}` with `{"priority": 2}`
- **THEN** the system SHALL update the priority and invalidate the group's Redis cache

#### Scenario: Update model mappings
- **WHEN** an authenticated admin sends PUT `/api/admin/groups/{group_id}/servers/{server_id}` with `{"model_mappings": {"claude-opus-4-6": "opus-custom"}}`
- **THEN** the system SHALL update the model_mappings and invalidate the group's Redis cache

### Requirement: Remove a server from a group
The system SHALL allow removing a server assignment from a group.

#### Scenario: Successful removal
- **WHEN** an authenticated admin sends DELETE `/api/admin/groups/{group_id}/servers/{server_id}`
- **THEN** the system SHALL delete the group_servers entry, invalidate the group's Redis cache, and return HTTP 204

### Requirement: Reorder server priorities in a group
The system SHALL allow reordering all server priorities in a group in a single request.

#### Scenario: Reorder priorities
- **WHEN** an authenticated admin sends PUT `/api/admin/groups/{group_id}/servers/reorder` with `{"server_ids": ["uuid-a", "uuid-b", "uuid-c"]}`
- **THEN** the system SHALL set priorities to 1, 2, 3 respectively (based on array order) and invalidate the group's Redis cache
