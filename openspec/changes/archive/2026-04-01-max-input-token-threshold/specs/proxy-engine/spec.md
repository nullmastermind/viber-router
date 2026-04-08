## MODIFIED Requirements

### Requirement: Streaming (SSE) passthrough
The system SHALL support streaming responses. When the upstream server returns a streaming SSE response, the system SHALL stream it directly to the client. When TTFT auto-switch is enabled and the current server is not the last in the waterfall, the system SHALL detect SSE responses in the proxy handler (before delegating to build_response) to enable TTFT timeout logic. When a count-tokens default server is configured and the request path is `/v1/messages/count_tokens`, the proxy handler SHALL attempt the default server before entering the failover waterfall, and SHALL skip the default server's `server_id` in the waterfall if the default attempt failed. **Before attempting each server, the proxy SHALL check if the server's circuit breaker is open (Redis key `cb:open:{group_id}:{server_id}` exists) and skip it if so. After each error (failover status code, connection error, or TTFT timeout), if the server has circuit breaker configured, the proxy SHALL increment the error counter and trip the breaker if threshold is reached.** When the request path is `/v1/messages` and the response is HTTP 200, the system SHALL wrap the forwarded stream to extract token usage from `message_start` and `message_delta` SSE events without modifying the stream content. **After reading the request body and extracting the model, if the group has a non-empty `allowed_models` list, the proxy SHALL validate the request model against the list. If the model is not in the list, or the request has no `model` field, the proxy SHALL return HTTP 403 with `permission_error` type and message "Your API key does not have permission to use the specified model." without contacting any upstream server. If the request was authenticated via a sub-key with a non-empty `key_allowed_models` list, the proxy SHALL additionally validate the model against the key's list. Errors from the `allowed_models` and `key_allowed_models` database queries SHALL be propagated (not silently swallowed) so that DB failures result in a request error rather than a security bypass.** **Before attempting each server in the failover waterfall, after the circuit breaker check, the proxy SHALL check if the server has rate limiting configured (`max_requests` and `rate_window_seconds` both non-null). If configured, the proxy SHALL check the Redis counter `rl:{group_id}:{server_id}`. If the count >= `max_requests`, the proxy SHALL skip this server. If the count < `max_requests`, the proxy SHALL INCR the counter (setting TTL to `rate_window_seconds` if the key is new) before sending the request. If Redis is unavailable during the rate limit check, the proxy SHALL proceed (fail open). When all servers in the chain are exhausted and at least one was skipped due to rate limiting, the proxy SHALL return HTTP 429 with `rate_limit_error` type and message "Rate limit exceeded".** **For requests to `/v1/messages` endpoint, before forwarding to the upstream server, the proxy SHALL merge the client-provided system prompt with the server's configured system_prompt (if any) using the following hybrid strategy: (1) If the client system is an array with cache_control metadata, merge the server_prompt into the last block's text field while preserving the cache_control; (2) If the client system is a string or an array without cache_control, normalize both to strings and concatenate with "\n\n" separator; (3) If only the server has a system_prompt, use it as-is; (4) If only the client has a system prompt, passthrough unchanged.** **Before entering the failover waterfall, the proxy SHALL estimate input tokens from the request body by: (1) parsing the body as JSON, (2) removing content objects where `"type" == "image"` from each message's content array, (3) serializing the filtered JSON to a string, (4) dividing the byte length by 4. The estimate SHALL be computed once and reused for all servers. If JSON parsing fails, the estimate is absent. For each server in the failover waterfall, after the rate limit check, if the server has `max_input_tokens` set (non-null) and the estimated token count is strictly greater than `max_input_tokens`, the proxy SHALL skip that server and continue to the next.**

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

#### Scenario: Token usage extraction from streaming response
- **WHEN** the request path is `/v1/messages` and the upstream returns a streaming HTTP 200 response
- **THEN** the system SHALL wrap the stream to extract `input_tokens` from `message_start` and `output_tokens` from `message_delta` events, forwarding all bytes unchanged to the client

#### Scenario: Token usage extraction from non-streaming response
- **WHEN** the request path is `/v1/messages` and the upstream returns a non-streaming HTTP 200 response
- **THEN** the system SHALL parse the response JSON to extract `usage.input_tokens` and `usage.output_tokens` before returning the response to the client

#### Scenario: Model not in group allowed list
- **WHEN** a group has `allowed_models: ["claude-sonnet-4-20250514", "gpt-4o"]` and the request body contains `"model": "claude-opus-4-20250514"`
- **THEN** the system SHALL return HTTP 403 with `{"type": "error", "error": {"type": "permission_error", "message": "Your API key does not have permission to use the specified model."}}` without contacting any upstream server

#### Scenario: Model allowed — pass-through to failover
- **WHEN** a group has `allowed_models: ["claude-sonnet-4-20250514"]` and the request body contains `"model": "claude-sonnet-4-20250514"`
- **THEN** the system SHALL proceed to the failover waterfall normally

#### Scenario: Empty allowed list — pass-through
- **WHEN** a group has an empty `allowed_models` list and the request body contains any model name
- **THEN** the system SHALL proceed to the failover waterfall normally (no model restriction)

#### Scenario: No model field in request — group has allowed models
- **WHEN** a group has a non-empty `allowed_models` list and the request body has no `model` field
- **THEN** the system SHALL return HTTP 403 with `permission_error`

#### Scenario: Key-level model restriction
- **WHEN** a sub-key has `key_allowed_models: ["claude-sonnet-4-20250514"]` and the request body contains `"model": "gpt-4o"` (which is in the group's allowed list but not the key's)
- **THEN** the system SHALL return HTTP 403 with `permission_error`

#### Scenario: Key with empty allowed list inherits group
- **WHEN** a sub-key has an empty `key_allowed_models` list and the group has `allowed_models: ["claude-sonnet-4-20250514", "gpt-4o"]`
- **THEN** the system SHALL allow any model in the group's allowed list

#### Scenario: Server rate-limited — skipped in waterfall
- **WHEN** a server has `max_requests=100, rate_window_seconds=60` and the Redis counter `rl:{group_id}:{server_id}` shows count >= 100
- **THEN** the system SHALL skip this server without sending a request and continue to the next server

#### Scenario: Server under rate limit — increment and proceed
- **WHEN** a server has `max_requests=100, rate_window_seconds=60` and the Redis counter shows count < 100
- **THEN** the system SHALL INCR the counter, set TTL if new key, and proceed to send the request

#### Scenario: Rate limit not configured — no rate check
- **WHEN** a server has `max_requests=NULL` and `rate_window_seconds=NULL`
- **THEN** the system SHALL NOT perform any rate limit Redis operations for this server

#### Scenario: Redis unavailable during rate limit check — fail open
- **WHEN** a server has rate limiting configured but Redis connection fails during the check
- **THEN** the system SHALL proceed to send the request (fail open)

#### Scenario: All servers rate-limited — distinct 429 error
- **WHEN** all servers in the chain are skipped due to rate limiting (no server attempted)
- **THEN** the system SHALL return HTTP 429 with `{"type": "error", "error": {"type": "rate_limit_error", "message": "Rate limit exceeded"}}`

#### Scenario: Proxy handler emits uptime check entries
- **WHEN** the proxy handler emits uptime entries
- **THEN** a `request_id` UUID SHALL be generated at the start of each request and shared across all server attempts

#### Scenario: Uptime entry emitted on successful response
- **WHEN** a server returns HTTP 200
- **THEN** the proxy SHALL emit an UptimeCheckEntry with the server's status_code=200 and latency_ms before returning the response

#### Scenario: Uptime entry emitted on failover
- **WHEN** a server returns a failover status code (e.g., 429) and the proxy continues to the next server
- **THEN** the proxy SHALL emit an UptimeCheckEntry with the server's status_code and latency_ms before trying the next server

#### Scenario: Uptime entry emitted on connection error
- **WHEN** a server connection fails
- **THEN** the proxy SHALL emit an UptimeCheckEntry with status_code=0 and the elapsed latency_ms

#### Scenario: Uptime entry emitted on TTFT timeout
- **WHEN** a server's first chunk exceeds the TTFT timeout
- **THEN** the proxy SHALL emit an UptimeCheckEntry with status_code=0 and the elapsed latency_ms

#### Scenario: All attempts share request_id
- **WHEN** a proxy request tries multiple servers
- **THEN** all emitted UptimeCheckEntry records SHALL share the same request_id UUID

#### Scenario: System prompt merge — client string + server string
- **WHEN** the request path is `/v1/messages`, client sends `"system": "You are a coding assistant"`, and the server has `system_prompt: "Always respond in Vietnamese"`
- **THEN** the proxy SHALL merge to `"system": "You are a coding assistant\n\nAlways respond in Vietnamese"` before forwarding

#### Scenario: System prompt merge — client array with cache_control + server string
- **WHEN** the request path is `/v1/messages`, client sends `"system": [{"type": "text", "text": "Block 1", "cache_control": {"type": "ephemeral"}}]`, and the server has `system_prompt: "Always respond in Vietnamese"`
- **THEN** the proxy SHALL merge to `"system": [{"type": "text", "text": "Block 1\n\nAlways respond in Vietnamese", "cache_control": {"type": "ephemeral"}}]` before forwarding

#### Scenario: System prompt merge — client array without cache_control + server string
- **WHEN** the request path is `/v1/messages`, client sends `"system": [{"type": "text", "text": "Block 1"}]` (no cache_control), and the server has `system_prompt: "Always respond in Vietnamese"`
- **THEN** the proxy SHALL normalize to string and merge to `"system": "Block 1\n\nAlways respond in Vietnamese"` before forwarding

#### Scenario: System prompt merge — only server has system_prompt
- **WHEN** the request path is `/v1/messages`, client sends no `system` field, and the server has `system_prompt: "Always respond in Vietnamese"`
- **THEN** the proxy SHALL set `"system": "Always respond in Vietnamese"` before forwarding

#### Scenario: System prompt merge — only client has system
- **WHEN** the request path is `/v1/messages`, client sends `"system": "You are helpful"`, and the server has no system_prompt (null)
- **THEN** the proxy SHALL passthrough `"system": "You are helpful"` unchanged

#### Scenario: System prompt merge — neither has system
- **WHEN** the request path is `/v1/messages`, client sends no `system` field, and the server has no system_prompt (null)
- **THEN** the proxy SHALL not add a `system` field to the request

#### Scenario: System prompt not merged on non-messages endpoints
- **WHEN** the request path is `/v1/messages/count_tokens` (not `/v1/messages`), client sends `"system": "You are helpful"`, and the server has `system_prompt: "Always respond in Vietnamese"`
- **THEN** the proxy SHALL NOT merge system prompts and SHALL forward the request unchanged

#### Scenario: Server token threshold exceeded — skipped
- **WHEN** a server has `max_input_tokens=30000` and the estimated input token count is 32000
- **THEN** the proxy SHALL skip this server without sending a request and continue to the next server in the waterfall

#### Scenario: Server token threshold not exceeded — proceed
- **WHEN** a server has `max_input_tokens=30000` and the estimated input token count is 28000
- **THEN** the proxy SHALL proceed with the server normally (circuit breaker and rate limit checks still apply)

#### Scenario: Server has no token threshold — no token check
- **WHEN** a server has `max_input_tokens=NULL`
- **THEN** the proxy SHALL NOT skip this server based on token count
