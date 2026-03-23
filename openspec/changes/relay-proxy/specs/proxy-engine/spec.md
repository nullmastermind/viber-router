## ADDED Requirements

### Requirement: Proxy intercepts all /v1/* requests
The system SHALL accept any HTTP request to `/v1/*` paths and forward it to the appropriate upstream server based on the group identified by the `x-api-key` header.

#### Scenario: Valid request to /v1/messages
- **WHEN** a POST request is sent to `/v1/messages` with a valid `x-api-key` header
- **THEN** the system SHALL forward the request to the highest-priority upstream server in the group and return the upstream response to the client

#### Scenario: Valid request to /v1/models
- **WHEN** a GET request is sent to `/v1/models` with a valid `x-api-key` header
- **THEN** the system SHALL forward the request to the highest-priority upstream server and return the upstream response

#### Scenario: Valid request to any /v1/* path
- **WHEN** a request with any HTTP method is sent to any `/v1/*` path with a valid `x-api-key` header
- **THEN** the system SHALL forward the request preserving the original method, path, query parameters, headers (except x-api-key which is swapped), and body

### Requirement: Group identification via x-api-key header
The system SHALL identify the target group by looking up the `x-api-key` header value against group API keys.

#### Scenario: Valid API key
- **WHEN** a request has `x-api-key` matching an active group's API key
- **THEN** the system SHALL use that group's configuration for routing

#### Scenario: Unknown API key
- **WHEN** a request has `x-api-key` that does not match any group
- **THEN** the system SHALL return HTTP 401 with Anthropic-compatible error JSON: `{"type":"error","error":{"type":"authentication_error","message":"Invalid API key"}}`

#### Scenario: Missing x-api-key header
- **WHEN** a request to `/v1/*` has no `x-api-key` header
- **THEN** the system SHALL return HTTP 401 with Anthropic-compatible error JSON

#### Scenario: Inactive group
- **WHEN** a request has `x-api-key` matching a group with `is_active: false`
- **THEN** the system SHALL return HTTP 403 with Anthropic-compatible error JSON: `{"type":"error","error":{"type":"permission_error","message":"API key is disabled"}}`

### Requirement: Upstream API key substitution
The system SHALL replace the client's `x-api-key` header with the upstream server's API key before forwarding.

#### Scenario: Key swap on forward
- **WHEN** forwarding a request to upstream server S1 which has api_key "sk-ant-upstream-xxx"
- **THEN** the `x-api-key` header in the forwarded request SHALL be "sk-ant-upstream-xxx"

### Requirement: Model name transformation
The system SHALL transform the `model` field in the JSON request body according to the model_mappings configured for the specific group-server pair. If no mapping exists for the model name, the original name SHALL be passed through unchanged.

#### Scenario: Model name mapped
- **WHEN** the request body contains `"model": "claude-opus-4-6"` and the group-server mapping has `{"claude-opus-4-6": "my-opus"}`
- **THEN** the forwarded request body SHALL contain `"model": "my-opus"`

#### Scenario: Model name not in mapping
- **WHEN** the request body contains `"model": "claude-haiku-4-5"` and the group-server mapping has no entry for "claude-haiku-4-5"
- **THEN** the forwarded request body SHALL contain `"model": "claude-haiku-4-5"` unchanged

#### Scenario: No model mappings configured
- **WHEN** the group-server pair has empty or null model_mappings
- **THEN** the request body SHALL be forwarded unchanged

#### Scenario: Request has no model field
- **WHEN** the request body has no `model` field (e.g., GET /v1/models)
- **THEN** the request body SHALL be forwarded unchanged

### Requirement: Priority-based failover waterfall
The system SHALL try upstream servers in priority order (lowest priority number first). If a server returns a status code that is in the group's `failover_status_codes` set, the system SHALL try the next server. This continues until a server returns a non-failover status code or all servers are exhausted.

#### Scenario: First server succeeds
- **WHEN** server at priority 1 returns HTTP 200
- **THEN** the system SHALL return that response to the client without trying other servers

#### Scenario: First server fails with failover code
- **WHEN** server at priority 1 returns HTTP 429 and 429 is in failover_status_codes
- **THEN** the system SHALL try the server at priority 2

#### Scenario: First server fails with non-failover code
- **WHEN** server at priority 1 returns HTTP 400 and 400 is NOT in failover_status_codes
- **THEN** the system SHALL return that 400 response to the client without trying other servers

#### Scenario: Connection error triggers failover
- **WHEN** the connection to server at priority 1 fails (timeout, DNS error, connection refused)
- **THEN** the system SHALL try the server at priority 2

#### Scenario: All servers exhausted
- **WHEN** all servers in the group return failover status codes or connection errors
- **THEN** the system SHALL return HTTP 429 with `Retry-After: 30` header and body `{"type":"error","error":{"type":"overloaded_error","message":"All upstream servers unavailable"}}`

### Requirement: Streaming (SSE) passthrough
The system SHALL support streaming responses. When the upstream server returns a streaming SSE response, the system SHALL stream it directly to the client.

#### Scenario: Streaming response passthrough
- **WHEN** the client sends `"stream": true` and the upstream returns an SSE stream with HTTP 200
- **THEN** the system SHALL stream the SSE events to the client as they arrive

#### Scenario: Streaming failover before stream starts
- **WHEN** the client sends `"stream": true` and the upstream returns a failover status code
- **THEN** the system SHALL try the next server (failover works normally before streaming begins)

#### Scenario: Non-streaming response passthrough
- **WHEN** the client sends `"stream": false` or no stream field
- **THEN** the system SHALL return the complete JSON response from upstream

### Requirement: Response passthrough
The system SHALL return the upstream response to the client without modification. Status code, headers, and body SHALL be forwarded as-is.

#### Scenario: Successful response passthrough
- **WHEN** the upstream returns HTTP 200 with a JSON body
- **THEN** the client SHALL receive HTTP 200 with the identical JSON body

#### Scenario: Error response passthrough (non-failover)
- **WHEN** the upstream returns HTTP 400 with an error body and 400 is not a failover code
- **THEN** the client SHALL receive HTTP 400 with the identical error body
