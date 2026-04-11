## MODIFIED Requirements

### Requirement: Get a group by ID
The system SHALL return a single group's details including its assigned servers with priorities and model mappings, the count-tokens default server configuration, the list of allowed models, and the list of blocked user-agents.

#### Scenario: Group exists
- **WHEN** an authenticated admin sends GET `/api/admin/groups/{id}`
- **THEN** the system SHALL return HTTP 200 with the group object including its servers array, `count_tokens_server_id`, `count_tokens_model_mappings` fields, and `allowed_models` array

#### Scenario: Group not found
- **WHEN** an authenticated admin sends GET `/api/admin/groups/{id}` with a non-existent UUID
- **THEN** the system SHALL return HTTP 404
