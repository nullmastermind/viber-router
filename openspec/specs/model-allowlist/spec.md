## ADDED Requirements

### Requirement: List models
The system SHALL return a paginated list of models with optional name search. Pagination SHALL use page/limit query parameters.

#### Scenario: Paginated list
- **WHEN** an authenticated admin sends GET `/api/admin/models?page=1&limit=20`
- **THEN** the system SHALL return up to 20 models with total count, current page, and total pages

#### Scenario: Search by name
- **WHEN** an authenticated admin sends GET `/api/admin/models?search=claude`
- **THEN** the system SHALL return models whose name contains "claude" (case-insensitive)

### Requirement: Create a model
The system SHALL allow creating a new model with a unique name.

#### Scenario: Successful creation
- **WHEN** an authenticated admin sends POST `/api/admin/models` with `{"name": "claude-sonnet-4-20250514"}`
- **THEN** the system SHALL create the model and return HTTP 201 with the created model object

#### Scenario: Duplicate name
- **WHEN** an authenticated admin sends POST `/api/admin/models` with a name that already exists
- **THEN** the system SHALL return HTTP 409 with error message "Model already exists"

### Requirement: Delete a model
The system SHALL allow deleting a model only if no group references it.

#### Scenario: Successful deletion
- **WHEN** an authenticated admin sends DELETE `/api/admin/models/{id}` and no group has this model in its allowed list
- **THEN** the system SHALL delete the model and return HTTP 204

#### Scenario: Model in use
- **WHEN** an authenticated admin sends DELETE `/api/admin/models/{id}` and at least one group has this model in its allowed list
- **THEN** the system SHALL return HTTP 409 with error message listing the group names that use this model

### Requirement: List group allowed models
The system SHALL return the list of allowed models for a specific group.

#### Scenario: Group has allowed models
- **WHEN** an authenticated admin sends GET `/api/admin/groups/{id}/allowed-models`
- **THEN** the system SHALL return the list of model objects assigned to this group

#### Scenario: Group has no allowed models
- **WHEN** an authenticated admin sends GET `/api/admin/groups/{id}/allowed-models` and the group has no allowed models configured
- **THEN** the system SHALL return an empty array

### Requirement: Add model to group allowed list
The system SHALL allow adding a model to a group's allowed list, either by selecting an existing model or creating a new one inline.

#### Scenario: Assign existing model by ID
- **WHEN** an authenticated admin sends POST `/api/admin/groups/{id}/allowed-models` with `{"model_id": "<uuid>"}`
- **THEN** the system SHALL add the model to the group's allowed list (upsert — safe to call if already assigned), invalidate the group's Redis cache, and return HTTP 201

#### Scenario: Create and assign new model by name
- **WHEN** an authenticated admin sends POST `/api/admin/groups/{id}/allowed-models` with `{"name": "gpt-4o"}`
- **THEN** the system SHALL create the model in the master list (if not exists), add it to the group's allowed list (upsert), invalidate the group's Redis cache, and return HTTP 201

#### Scenario: Duplicate assignment
- **WHEN** an authenticated admin sends POST `/api/admin/groups/{id}/allowed-models` with a model already in the group's allowed list
- **THEN** the system SHALL upsert (no error), invalidate the group's Redis cache, and return HTTP 201

### Requirement: Remove model from group allowed list
The system SHALL allow removing a model from a group's allowed list. This SHALL cascade-delete the model from all sub-keys of the group.

#### Scenario: Successful removal
- **WHEN** an authenticated admin sends DELETE `/api/admin/groups/{id}/allowed-models/{model_id}`
- **THEN** the system SHALL remove the model from the group's allowed list, cascade-delete it from all sub-keys of this group, invalidate the group's Redis cache, and return HTTP 204

#### Scenario: Model not in group
- **WHEN** an authenticated admin sends DELETE `/api/admin/groups/{id}/allowed-models/{model_id}` and the model is not in the group's allowed list
- **THEN** the system SHALL return HTTP 404

### Requirement: List key allowed models
The system SHALL return the list of allowed models for a specific sub-key.

#### Scenario: Key has allowed models
- **WHEN** an authenticated admin sends GET `/api/admin/groups/{id}/keys/{key_id}/allowed-models`
- **THEN** the system SHALL return the list of model objects assigned to this key

#### Scenario: Key has no allowed models
- **WHEN** an authenticated admin sends GET `/api/admin/groups/{id}/keys/{key_id}/allowed-models` and the key has no allowed models configured
- **THEN** the system SHALL return an empty array

### Requirement: Add model to key allowed list
The system SHALL allow adding a model to a sub-key's allowed list. The model MUST already be in the parent group's allowed list.

#### Scenario: Assign model from group's allowed list
- **WHEN** an authenticated admin sends POST `/api/admin/groups/{id}/keys/{key_id}/allowed-models` with `{"model_id": "<uuid>"}` and the model is in the group's allowed list
- **THEN** the system SHALL add the model to the key's allowed list, invalidate the group's Redis cache, and return HTTP 201

#### Scenario: Model not in group's allowed list
- **WHEN** an authenticated admin sends POST `/api/admin/groups/{id}/keys/{key_id}/allowed-models` with `{"model_id": "<uuid>"}` and the model is NOT in the group's allowed list
- **THEN** the system SHALL return HTTP 400 with error message "Model is not in the group's allowed list"

#### Scenario: Group has no allowed models configured
- **WHEN** an authenticated admin sends POST `/api/admin/groups/{id}/keys/{key_id}/allowed-models` with `{"model_id": "<uuid>"}` and the group has an empty allowed models list
- **THEN** the system SHALL return HTTP 400 with error message "Group has no allowed models configured. Configure group-level allowed models first."

#### Scenario: Duplicate assignment
- **WHEN** an authenticated admin sends POST `/api/admin/groups/{id}/keys/{key_id}/allowed-models` with a model already in the key's allowed list
- **THEN** the system SHALL return HTTP 409 with error message "Model already assigned to this key"

### Requirement: Remove model from key allowed list
The system SHALL allow removing a model from a sub-key's allowed list.

#### Scenario: Successful removal
- **WHEN** an authenticated admin sends DELETE `/api/admin/groups/{id}/keys/{key_id}/allowed-models/{model_id}`
- **THEN** the system SHALL remove the model from the key's allowed list, invalidate the group's Redis cache, and return HTTP 204

#### Scenario: Model not in key
- **WHEN** an authenticated admin sends DELETE `/api/admin/groups/{id}/keys/{key_id}/allowed-models/{model_id}` and the model is not in the key's allowed list
- **THEN** the system SHALL return HTTP 404
