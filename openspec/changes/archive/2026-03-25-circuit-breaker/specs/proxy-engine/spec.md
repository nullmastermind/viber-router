## MODIFIED Requirements

### Requirement: Streaming (SSE) passthrough
The system SHALL support streaming responses. When the upstream server returns a streaming SSE response, the system SHALL stream it directly to the client. When TTFT auto-switch is enabled and the current server is not the last in the waterfall, the system SHALL detect SSE responses in the proxy handler (before delegating to build_response) to enable TTFT timeout logic. When a count-tokens default server is configured and the request path is `/v1/messages/count_tokens`, the proxy handler SHALL attempt the default server before entering the failover waterfall, and SHALL skip the default server's `server_id` in the waterfall if the default attempt failed. **Before attempting each server, the proxy SHALL check if the server's circuit breaker is open (Redis key `cb:open:{group_id}:{server_id}` exists) and skip it if so. After each error (failover status code, connection error, or TTFT timeout), if the server has circuit breaker configured, the proxy SHALL increment the error counter and trip the breaker if threshold is reached.**

#### Scenario: Streaming response passthrough
- **WHEN** the client sends `"stream": true` and the upstream returns an SSE stream with HTTP 200
- **THEN** the system SHALL stream the SSE events to the client as they arrive

#### Scenario: Streaming failover before stream starts
- **WHEN** the client sends `"stream": true` and the upstream returns a failover status code
- **THEN** the system SHALL try the next server (failover works normally before streaming begins)

#### Scenario: Non-streaming response passthrough
- **WHEN** the client sends `"stream": false` or no stream field
- **THEN** the system SHALL return the complete JSON response from upstream

#### Scenario: SSE with TTFT timeout enabled — first chunk within threshold
- **WHEN** TTFT auto-switch is enabled, the server is not the last in the waterfall, and the first SSE chunk arrives within `ttft_timeout_ms`
- **THEN** the system SHALL include the first chunk in the response stream and continue streaming normally

#### Scenario: SSE with TTFT timeout enabled — timeout exceeded
- **WHEN** TTFT auto-switch is enabled, the server is not the last in the waterfall, and no SSE chunk arrives within `ttft_timeout_ms`
- **THEN** the system SHALL close the upstream connection and try the next server in the failover waterfall

#### Scenario: SSE with TTFT disabled or last server
- **WHEN** TTFT auto-switch is disabled (NULL) or the current server is the last in the waterfall
- **THEN** the system SHALL use the existing build_response path unchanged (wait indefinitely for stream data)

#### Scenario: Count-tokens default server — pre-chain attempt
- **WHEN** a group has `count_tokens_server_id` configured and the request path is `/v1/messages/count_tokens`
- **THEN** the system SHALL attempt the default server before entering the failover waterfall

#### Scenario: Count-tokens default server — skip in chain after failure
- **WHEN** the default count-tokens server fails and the failover waterfall contains a server with the same `server_id`
- **THEN** the system SHALL skip that server in the waterfall to avoid a redundant retry

#### Scenario: Server with open circuit breaker — skipped
- **WHEN** a server in the failover chain has `cb:open:{group_id}:{server_id}` key existing in Redis
- **THEN** the system SHALL skip this server without sending a request and continue to the next server

#### Scenario: Server error increments circuit breaker counter
- **WHEN** a server with `cb_max_failures=5` returns a failover status code or connection error
- **THEN** the system SHALL INCR `cb:err:{group_id}:{server_id}` in Redis and if count >= 5, trip the circuit breaker

#### Scenario: Circuit breaker not configured — no circuit check
- **WHEN** a server has `cb_max_failures=NULL` and returns an error
- **THEN** the system SHALL NOT perform any circuit breaker Redis operations for this server
