## MODIFIED Requirements

### Requirement: Streaming (SSE) passthrough
The system SHALL support streaming responses. When the upstream server returns a streaming SSE response, the system SHALL stream it directly to the client. When TTFT auto-switch is enabled and the current server is not the last in the waterfall, the system SHALL detect SSE responses in the proxy handler (before delegating to build_response) to enable TTFT timeout logic.

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
