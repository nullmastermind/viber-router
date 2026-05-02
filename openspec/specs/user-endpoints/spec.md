## Purpose
TBD

## Requirements
### Requirement: User endpoint persistence
The system SHALL persist user-managed custom endpoints in a `user_endpoints` table associated with `group_keys.id`. Each endpoint SHALL include a UUID `id`, `group_key_id`, `name`, `base_url`, `api_key`, JSON object `model_mappings`, optional `quota_url`, optional JSON object `quota_headers`, `priority_mode` constrained to `priority` or `fallback`, `is_enabled`, `created_at`, and `updated_at`.

#### Scenario: Endpoint row created with defaults
- **WHEN** a user endpoint is created without explicit model mappings, priority mode, or enabled state
- **THEN** the system SHALL store `model_mappings` as `{}`, `priority_mode` as `fallback`, and `is_enabled` as `true`

#### Scenario: Endpoint deleted with owning key
- **WHEN** a `group_keys` row is deleted
- **THEN** all `user_endpoints` rows for that `group_key_id` SHALL be deleted by cascade

### Requirement: User endpoint public management API
The system SHALL expose public CRUD routes for user endpoints authenticated by the same `key=<sub-key>` query parameter used by the public usage endpoint. The routes SHALL include `GET /api/public/user-endpoints`, `POST /api/public/user-endpoints`, `PATCH /api/public/user-endpoints/{id}`, and `DELETE /api/public/user-endpoints/{id}`.

#### Scenario: List endpoints for valid sub-key
- **WHEN** a request is made to `GET /api/public/user-endpoints?key=<valid-active-sub-key>`
- **THEN** the system SHALL return only user endpoints owned by that sub-key's `group_key_id`

#### Scenario: Create endpoint for valid sub-key
- **WHEN** a request is made to `POST /api/public/user-endpoints?key=<valid-active-sub-key>` with valid endpoint fields
- **THEN** the system SHALL create the endpoint for that sub-key's `group_key_id` and return the created endpoint

#### Scenario: Update owned endpoint
- **WHEN** a request is made to `PATCH /api/public/user-endpoints/{id}?key=<valid-active-sub-key>` for an endpoint owned by that sub-key
- **THEN** the system SHALL update only the provided fields, including enabled toggles, and return the updated endpoint

#### Scenario: Delete owned endpoint
- **WHEN** a request is made to `DELETE /api/public/user-endpoints/{id}?key=<valid-active-sub-key>` for an endpoint owned by that sub-key
- **THEN** the system SHALL delete the endpoint from the database

#### Scenario: Invalid or inactive sub-key
- **WHEN** any user endpoint management route is called with a missing, invalid, or inactive sub-key
- **THEN** the system SHALL reject the request using the same public-key error behavior as `GET /api/public/usage`

#### Scenario: Cross-key endpoint access denied
- **WHEN** a sub-key attempts to patch or delete an endpoint belonging to another `group_key_id`
- **THEN** the system SHALL NOT modify that endpoint and SHALL return an error response

### Requirement: User endpoint limit
The system SHALL enforce a maximum of 10 user endpoints per sub-key.

#### Scenario: Create within limit
- **WHEN** a sub-key has fewer than 10 user endpoints and submits a valid create request
- **THEN** the system SHALL create the endpoint

#### Scenario: Create beyond limit
- **WHEN** a sub-key already has 10 user endpoints and submits a create request
- **THEN** the system SHALL reject the request with a clear max-limit error and SHALL NOT create an additional endpoint

### Requirement: User endpoint validation
The system SHALL validate required endpoint fields and JSON object fields before persisting changes. `name`, `base_url`, and `api_key` SHALL be required for creation. `model_mappings` and `quota_headers`, when provided, SHALL be JSON objects. `priority_mode`, when provided, SHALL be either `priority` or `fallback`.

#### Scenario: Required fields missing
- **WHEN** a create request omits `name`, `base_url`, or `api_key`
- **THEN** the system SHALL reject the request and SHALL NOT create a row

#### Scenario: Invalid model mappings
- **WHEN** a create or update request provides `model_mappings` that is not a JSON object
- **THEN** the system SHALL reject the request and SHALL NOT persist the invalid mapping

#### Scenario: Invalid quota headers
- **WHEN** a create or update request provides `quota_headers` that is not a JSON object
- **THEN** the system SHALL reject the request and SHALL NOT persist the invalid headers

#### Scenario: Invalid priority mode
- **WHEN** a create or update request provides `priority_mode` other than `priority` or `fallback`
- **THEN** the system SHALL reject the request and SHALL NOT persist the invalid mode

### Requirement: User endpoint model compatibility
The system SHALL consider a user endpoint model-compatible when it is enabled and either its `model_mappings` object is empty or the request model appears as a key in `model_mappings`. If a key is present, the request body model SHALL be transformed to the mapped target model before forwarding to that endpoint.

#### Scenario: Empty mappings accept all models
- **WHEN** a user endpoint has `model_mappings = {}` and the request body contains any model
- **THEN** the endpoint SHALL be considered model-compatible

#### Scenario: Mapping key accepts source model
- **WHEN** a user endpoint has `model_mappings = {"claude-3-5-sonnet": "provider-sonnet"}` and the request model is `claude-3-5-sonnet`
- **THEN** the endpoint SHALL be considered model-compatible and the forwarded body SHALL use `provider-sonnet`

#### Scenario: Mapping without source model rejects endpoint
- **WHEN** a user endpoint has `model_mappings = {"claude-3-5-sonnet": "provider-sonnet"}` and the request model is `gpt-4o`
- **THEN** the endpoint SHALL NOT be attempted for that request

#### Scenario: Disabled endpoint skipped
- **WHEN** a user endpoint has `is_enabled = false`
- **THEN** the endpoint SHALL NOT be attempted by proxy routing regardless of model mappings

### Requirement: User endpoint cache
The system SHALL provide cache helpers for user endpoints using Redis key `user_eps:{group_key_id}` with a 300-second TTL. Mutations to a sub-key's user endpoints SHALL invalidate or refresh that key.

#### Scenario: Cached endpoints loaded
- **WHEN** proxy routing needs user endpoints for a `group_key_id` and a valid cache entry exists
- **THEN** the system SHALL use the cached endpoints instead of querying the database

#### Scenario: Cache miss loads database
- **WHEN** proxy routing needs user endpoints and no valid cache entry exists
- **THEN** the system SHALL query the database, cache the result for 300 seconds, and use it for routing

#### Scenario: Mutation invalidates cache
- **WHEN** a user endpoint is created, updated, toggled, or deleted
- **THEN** the system SHALL invalidate or refresh `user_eps:{group_key_id}` before future routing decisions depend on it

### Requirement: User endpoint quota and usage summary
The system SHALL provide quota data, when `quota_url` is configured, and recent usage statistics for each user endpoint in public user endpoint responses and public usage responses. Usage statistics SHALL cover the last 30 days and SHALL be grouped per endpoint and model.

#### Scenario: Quota URL configured
- **WHEN** a listed user endpoint has `quota_url` and optional `quota_headers`
- **THEN** the system SHALL attempt to fetch quota data using those headers and include quota information in the endpoint response

#### Scenario: Quota URL absent
- **WHEN** a listed user endpoint has no `quota_url`
- **THEN** the endpoint response SHALL indicate no quota data rather than failing the list request

#### Scenario: Endpoint usage included
- **WHEN** a user endpoint has token usage logs in the last 30 days
- **THEN** the endpoint response SHALL include per-model usage statistics for that endpoint
