## ADDED Requirements

### Requirement: Extract token usage from streaming SSE responses
When the proxy forwards a streaming `/v1/messages` response with HTTP 200, the system SHALL extract token usage from SSE events without modifying the forwarded stream. The system SHALL read `input_tokens` from the `message_start` event and `output_tokens` from the `message_delta` event.

#### Scenario: Successful streaming response with usage
- **WHEN** the upstream returns a streaming HTTP 200 response for `/v1/messages` containing a `message_start` event with `usage.input_tokens: 100` and a `message_delta` event with `usage.output_tokens: 50`
- **THEN** the system SHALL extract `input_tokens: 100` and `output_tokens: 50` and emit a usage log entry

#### Scenario: Streaming response with cache tokens
- **WHEN** the `message_start` event contains `usage.cache_creation_input_tokens: 20` and `usage.cache_read_input_tokens: 30`
- **THEN** the system SHALL extract those values alongside `input_tokens` and `output_tokens`

#### Scenario: Stream ends without message_delta (incomplete)
- **WHEN** the stream terminates (client disconnect or upstream error) before a `message_delta` event is received
- **THEN** the system SHALL NOT emit a usage log entry

#### Scenario: SSE event spans multiple byte chunks
- **WHEN** an SSE event is split across two or more byte chunks from the upstream
- **THEN** the system SHALL buffer partial event data and parse the complete event once the `\n\n` delimiter is received

#### Scenario: Stream forwarding is unaffected
- **WHEN** the system extracts usage from SSE events
- **THEN** every byte chunk SHALL be forwarded to the client unchanged and without additional latency

### Requirement: Extract token usage from non-streaming responses
When the proxy returns a non-streaming `/v1/messages` response with HTTP 200, the system SHALL parse the JSON response body to extract the top-level `usage` object.

#### Scenario: Successful non-streaming response with usage
- **WHEN** the upstream returns a non-streaming HTTP 200 response for `/v1/messages` with `usage: { input_tokens: 100, output_tokens: 50 }`
- **THEN** the system SHALL extract `input_tokens: 100` and `output_tokens: 50` and emit a usage log entry

#### Scenario: Non-streaming response with cache tokens
- **WHEN** the response body contains `usage.cache_creation_input_tokens` and `usage.cache_read_input_tokens`
- **THEN** the system SHALL extract those values alongside `input_tokens` and `output_tokens`

#### Scenario: Non-streaming response missing usage
- **WHEN** the response body does not contain a `usage` object or is missing `input_tokens` or `output_tokens`
- **THEN** the system SHALL NOT emit a usage log entry

### Requirement: Skip non-applicable requests
The system SHALL only extract token usage for `/v1/messages` requests that return HTTP 200.

#### Scenario: Non-200 response
- **WHEN** the upstream returns a non-200 status code for `/v1/messages`
- **THEN** the system SHALL NOT attempt to extract token usage

#### Scenario: Count-tokens request
- **WHEN** the request path is `/v1/messages/count_tokens`
- **THEN** the system SHALL NOT attempt to extract token usage

#### Scenario: Non-messages endpoint
- **WHEN** the request path is not `/v1/messages`
- **THEN** the system SHALL NOT attempt to extract token usage
