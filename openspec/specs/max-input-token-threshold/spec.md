## Requirements

### Requirement: Token estimation strips image blocks and divides by 4
Before entering the failover waterfall, the proxy SHALL estimate the number of input tokens from the request body using the following algorithm: (1) parse the body as JSON; (2) for each message in the `messages` array, remove content blocks where `"type" == "image"`; (3) serialize the filtered JSON back to a string; (4) divide the byte length of the filtered string by 4. If JSON parsing fails, the estimated token count SHALL be treated as absent (None) and the threshold check SHALL be skipped for all servers.

#### Scenario: Estimation from text-only request
- **WHEN** the request body contains only text content blocks (no images)
- **THEN** the proxy SHALL compute estimated_tokens as the full filtered JSON string length divided by 4

#### Scenario: Estimation strips image content blocks
- **WHEN** the request body contains messages with mixed text and image content blocks
- **THEN** the proxy SHALL remove all content objects where `"type" == "image"` before measuring byte length, so that base64 image data does not inflate the estimate

#### Scenario: Non-JSON body — skip estimation
- **WHEN** the request body cannot be parsed as valid JSON
- **THEN** the proxy SHALL treat estimated_tokens as absent and SHALL NOT skip any server based on the token threshold

### Requirement: Proxy skips servers exceeding max_input_tokens threshold
During the failover waterfall, for each server that has `max_input_tokens` set (non-null), the proxy SHALL compare the estimated token count to the threshold. If the estimated token count is greater than the server's `max_input_tokens`, the proxy SHALL skip that server and continue to the next. If `max_input_tokens` is NULL or no estimate is available, the proxy SHALL not skip the server based on this check.

#### Scenario: Server threshold exceeded — server skipped
- **WHEN** a server has `max_input_tokens=30000` and the estimated token count is 32000
- **THEN** the proxy SHALL skip this server without sending a request and continue to the next server in the waterfall

#### Scenario: Server threshold not exceeded — proceed
- **WHEN** a server has `max_input_tokens=30000` and the estimated token count is 28000
- **THEN** the proxy SHALL NOT skip this server based on the token threshold and SHALL proceed with the normal circuit breaker and rate limit checks

#### Scenario: Server threshold exactly at limit — proceed
- **WHEN** a server has `max_input_tokens=30000` and the estimated token count is exactly 30000
- **THEN** the proxy SHALL NOT skip this server (threshold is exceeded only when strictly greater than)

#### Scenario: max_input_tokens NULL — no skip
- **WHEN** a server has `max_input_tokens=NULL`
- **THEN** the proxy SHALL NOT skip this server based on token count regardless of request size

#### Scenario: Estimation absent — no token-based skip
- **WHEN** the estimated token count is absent (e.g., non-JSON body) and a server has `max_input_tokens=30000`
- **THEN** the proxy SHALL NOT skip this server based on the token threshold
