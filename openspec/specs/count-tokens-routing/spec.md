## Purpose
TBD

## Requirements
### Requirement: Count-tokens default server selection
When a group has `count_tokens_server_id` configured and a request targets `/v1/messages/count_tokens`, the system SHALL try the configured default server before the normal failover waterfall.

#### Scenario: Default server succeeds
- **WHEN** a group has `count_tokens_server_id` set to server S and a request hits `/v1/messages/count_tokens`
- **THEN** the system SHALL forward the request to server S first and return its response if successful (HTTP 200)

#### Scenario: Default server not configured — normal routing
- **WHEN** a group has `count_tokens_server_id` set to NULL and a request hits `/v1/messages/count_tokens`
- **THEN** the system SHALL route the request through the normal failover waterfall with no changes

#### Scenario: Non-count-tokens path — normal routing
- **WHEN** a group has `count_tokens_server_id` configured and a request hits any path other than `/v1/messages/count_tokens`
- **THEN** the system SHALL route the request through the normal failover waterfall with no changes

### Requirement: Count-tokens default server failover
When the default count-tokens server fails with a failover status code or connection error, the system SHALL fall back to the normal failover waterfall, skipping the default server if it also appears in the chain.

#### Scenario: Default server returns failover status code
- **WHEN** the default server returns a status code in the group's `failover_status_codes` (e.g., 429)
- **THEN** the system SHALL proceed to the normal failover waterfall, skipping any server with the same `server_id` as the default

#### Scenario: Default server connection error
- **WHEN** the default server is unreachable (connection error)
- **THEN** the system SHALL proceed to the normal failover waterfall, skipping any server with the same `server_id` as the default

#### Scenario: Default server returns non-failover error
- **WHEN** the default server returns a non-failover error (e.g., 400)
- **THEN** the system SHALL return the error response directly to the client without trying the failover chain

### Requirement: Count-tokens key resolution
The system SHALL resolve the API key for the default count-tokens server using the same priority as the normal failover chain: dynamic key via `short_id` first, then the server's default `api_key`.

#### Scenario: Dynamic key available for default server
- **WHEN** the client provides a dynamic key segment matching the default server's `short_id`
- **THEN** the system SHALL use the dynamic key for the default server request

#### Scenario: No dynamic key — use server default key
- **WHEN** no dynamic key matches the default server's `short_id` and the server has an `api_key` configured
- **THEN** the system SHALL use the server's `api_key`

#### Scenario: No key available — skip default server
- **WHEN** no dynamic key matches and the server has no `api_key`
- **THEN** the system SHALL skip the default server and proceed directly to the normal failover waterfall

### Requirement: Count-tokens model mapping
The system SHALL transform the `model` field in count-tokens requests using the group's `count_tokens_model_mappings` (not the per-server mappings from `group_servers`).

#### Scenario: Model mapping configured
- **WHEN** the group has `count_tokens_model_mappings: {"claude-opus-4-6": "my-opus"}` and the request body contains `"model": "claude-opus-4-6"`
- **THEN** the system SHALL transform the model to `"my-opus"` before forwarding to the default server

#### Scenario: No mapping for model
- **WHEN** the group has `count_tokens_model_mappings` but no entry for the request's model
- **THEN** the system SHALL forward the original model name unchanged

#### Scenario: Empty model mappings
- **WHEN** the group has `count_tokens_model_mappings: {}`
- **THEN** the system SHALL forward the request body unchanged
