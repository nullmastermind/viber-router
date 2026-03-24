## MODIFIED Requirements

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

### Requirement: Get a group by ID
The system SHALL return a single group's details including its assigned servers with priorities and model mappings, plus the count-tokens default server configuration.

#### Scenario: Group exists
- **WHEN** an authenticated admin sends GET `/api/admin/groups/{id}`
- **THEN** the system SHALL return HTTP 200 with the group object including its servers array and `count_tokens_server_id` and `count_tokens_model_mappings` fields
