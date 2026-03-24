## ADDED Requirements

### Requirement: Create a group
The system SHALL allow creating a new group with a name and optional failover_status_codes. An API key SHALL be auto-generated with the format `sk-vibervn-` followed by 24 random alphanumeric characters. The group SHALL default to `is_active: true`.

#### Scenario: Successful group creation
- **WHEN** an authenticated admin sends POST `/api/admin/groups` with `{"name": "Team Alpha", "failover_status_codes": [429, 500, 502, 503]}`
- **THEN** the system SHALL create the group with an auto-generated API key (format: `sk-vibervn-xxxxxxxxxxxxxxxxxxxx`) and return HTTP 201 with the created group object

#### Scenario: Default failover codes
- **WHEN** an authenticated admin sends POST `/api/admin/groups` with `{"name": "Team Beta"}` and no failover_status_codes
- **THEN** the system SHALL create the group with default failover_status_codes of `[429, 500, 502, 503]`

### Requirement: List groups with server-side pagination
The system SHALL return a paginated list of groups with support for search, filtering, and sorting. Pagination SHALL use page/limit query parameters.

#### Scenario: Paginated list
- **WHEN** an authenticated admin sends GET `/api/admin/groups?page=1&limit=20`
- **THEN** the system SHALL return up to 20 groups with total count, current page, and total pages

#### Scenario: Search by name
- **WHEN** an authenticated admin sends GET `/api/admin/groups?search=alpha`
- **THEN** the system SHALL return groups whose name contains "alpha" (case-insensitive)

#### Scenario: Filter by active status
- **WHEN** an authenticated admin sends GET `/api/admin/groups?is_active=true`
- **THEN** the system SHALL return only active groups

#### Scenario: Filter by server
- **WHEN** an authenticated admin sends GET `/api/admin/groups?server_id={uuid}`
- **THEN** the system SHALL return only groups that have the specified server assigned

#### Scenario: Sort by created_at
- **WHEN** an authenticated admin sends GET `/api/admin/groups?sort=created_at&order=desc`
- **THEN** the system SHALL return groups sorted by creation date descending

### Requirement: Get a group by ID
The system SHALL return a single group's details including its assigned servers with priorities and model mappings, plus the count-tokens default server configuration.

#### Scenario: Group exists
- **WHEN** an authenticated admin sends GET `/api/admin/groups/{id}`
- **THEN** the system SHALL return HTTP 200 with the group object including its servers array and `count_tokens_server_id` and `count_tokens_model_mappings` fields

#### Scenario: Group not found
- **WHEN** an authenticated admin sends GET `/api/admin/groups/{id}` with a non-existent UUID
- **THEN** the system SHALL return HTTP 404

### Requirement: Update a group
The system SHALL allow updating a group's name, failover_status_codes, is_active status, ttft_timeout_ms, count_tokens_server_id, and count_tokens_model_mappings. The Redis cache for this group SHALL be invalidated on update.

#### Scenario: Successful update
- **WHEN** an authenticated admin sends PUT `/api/admin/groups/{id}` with updated fields
- **THEN** the system SHALL update the group, invalidate its Redis cache, and return HTTP 200

#### Scenario: Deactivate a group
- **WHEN** an authenticated admin sends PUT `/api/admin/groups/{id}` with `{"is_active": false}`
- **THEN** the system SHALL set the group to inactive and invalidate its Redis cache

#### Scenario: Set count-tokens default server
- **WHEN** an authenticated admin sends PUT `/api/admin/groups/{id}` with `{"count_tokens_server_id": "<uuid>"}`
- **THEN** the system SHALL set the count-tokens default server and invalidate the group's Redis cache

#### Scenario: Clear count-tokens default server
- **WHEN** an authenticated admin sends PUT `/api/admin/groups/{id}` with `{"count_tokens_server_id": null}`
- **THEN** the system SHALL clear the count-tokens default server (set to NULL) and invalidate the group's Redis cache

#### Scenario: Set count-tokens model mappings
- **WHEN** an authenticated admin sends PUT `/api/admin/groups/{id}` with `{"count_tokens_model_mappings": {"claude-opus-4-6": "my-opus"}}`
- **THEN** the system SHALL update the count-tokens model mappings and invalidate the group's Redis cache

### Requirement: Delete a group
The system SHALL allow deleting a group. This SHALL also delete all group_servers entries for this group and invalidate the Redis cache.

#### Scenario: Successful deletion
- **WHEN** an authenticated admin sends DELETE `/api/admin/groups/{id}`
- **THEN** the system SHALL delete the group, its group_servers entries, and its Redis cache entry, and return HTTP 204

### Requirement: Regenerate group API key
The system SHALL allow regenerating a group's API key. The old key's cache entry SHALL be invalidated and a new cache entry created.

#### Scenario: Successful key regeneration
- **WHEN** an authenticated admin sends POST `/api/admin/groups/{id}/regenerate-key`
- **THEN** the system SHALL generate a new `sk-vibervn-*` key, invalidate the old key's Redis cache, and return HTTP 200 with the updated group

### Requirement: Bulk activate/deactivate groups
The system SHALL allow activating or deactivating multiple groups in a single request.

#### Scenario: Bulk activate
- **WHEN** an authenticated admin sends POST `/api/admin/groups/bulk/activate` with `{"ids": ["uuid1", "uuid2"]}`
- **THEN** the system SHALL set all specified groups to `is_active: true` and invalidate their Redis cache entries

#### Scenario: Bulk deactivate
- **WHEN** an authenticated admin sends POST `/api/admin/groups/bulk/deactivate` with `{"ids": ["uuid1", "uuid2"]}`
- **THEN** the system SHALL set all specified groups to `is_active: false` and invalidate their Redis cache entries

### Requirement: Bulk delete groups
The system SHALL allow deleting multiple groups in a single request.

#### Scenario: Bulk delete
- **WHEN** an authenticated admin sends POST `/api/admin/groups/bulk/delete` with `{"ids": ["uuid1", "uuid2"]}`
- **THEN** the system SHALL delete all specified groups, their group_servers entries, and their Redis cache entries, and return HTTP 200

### Requirement: Bulk assign server to groups
The system SHALL allow adding a server to multiple groups at once with a specified priority and optional model mappings.

#### Scenario: Bulk assign
- **WHEN** an authenticated admin sends POST `/api/admin/groups/bulk/assign-server` with `{"group_ids": ["uuid1", "uuid2"], "server_id": "uuid3", "priority": 99, "model_mappings": {}}`
- **THEN** the system SHALL add the server to each specified group with the given priority and model mappings, and invalidate their Redis cache entries
