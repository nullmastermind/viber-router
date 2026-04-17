## MODIFIED Requirements

### Requirement: Streaming (SSE) passthrough
The system SHALL support streaming responses. When the upstream server returns a streaming SSE response, the system SHALL stream it directly to the client. When TTFT auto-switch is enabled and the current server is not the last in the waterfall, the system SHALL detect SSE responses in the proxy handler (before delegating to build_response) to enable TTFT timeout logic. When a count-tokens default server is configured and the request path is `/v1/messages/count_tokens`, the proxy handler SHALL attempt the default server before entering the failover waterfall, and SHALL skip the default server's `server_id` in the waterfall if the default attempt failed. **Before attempting each server, the proxy SHALL check if the server's circuit breaker is open (Redis key `cb:open:{group_id}:{server_id}` exists) and skip it if so. After each error (failover status code, connection error, or TTFT timeout), if the server has circuit breaker configured, the proxy SHALL increment the error counter and trip the breaker if threshold is reached.** **After reading the request body and extracting the model, if the group has a non-empty `allowed_models` list, the proxy SHALL validate the request model against the list. If the model is not in the list, or the request has no `model` field, the proxy SHALL return HTTP 403 with `permission_error` type and message "Your API key does not have permission to use the specified model." without contacting any upstream server. If the request was authenticated via a sub-key with a non-empty `key_allowed_models` list, the proxy SHALL additionally validate the model against the key's list. Errors from the `allowed_models` and `key_allowed_models` database queries SHALL be propagated (not silently swallowed) so that DB failures result in a request error rather than a security bypass.** **Before attempting each server in the failover waterfall, after the circuit breaker check, the proxy SHALL check if the server has rate limiting configured (`max_requests` and `rate_window_seconds` both non-null). If configured, the proxy SHALL check the Redis counter `rl:{group_id}:{server_id}`. If the count >= `max_requests`, the proxy SHALL skip this server. If the count < `max_requests`, the proxy SHALL INCR the counter (setting TTL to `rate_window_seconds` if the key is new) before sending the request. If Redis is unavailable during the rate limit check, the proxy SHALL proceed (fail open). When all servers in the chain are exhausted and at least one was skipped due to rate limiting, the proxy SHALL return HTTP 429 with `rate_limit_error` type and message "Rate limit exceeded".** **For requests to `/v1/messages`, before forwarding to the upstream server, the proxy SHALL merge the client-provided system prompt with the server's configured system_prompt (if any) using the existing hybrid strategy targeting the top-level `system` field. For requests to paths starting with `/v1/chat/`, the proxy SHALL merge the server system_prompt into the `messages` array.** **Before entering the failover waterfall, the proxy SHALL estimate input tokens from the request body by: (1) parsing the body as JSON, (2) removing content objects where `"type" == "image"` from each message's content array, (3) serializing the filtered JSON to a string, (4) dividing the byte length by 4. The estimate SHALL be computed once and reused for all servers. If JSON parsing fails, the estimate is absent. For each server in the failover waterfall, after the rate limit check, if the server has `max_input_tokens` set (non-null) and the estimated token count is strictly greater than `max_input_tokens`, the proxy SHALL skip that server and continue to the next.** **Error responses SHALL use the format matching the request path: Anthropic format (`{"type":"error","error":{"type":...,"message":...}}`) for `/v1/messages` and related paths; OpenAI format (`{"error":{"message":...,"type":...,"param":null,"code":null}}`) for paths starting with `/v1/chat/`. All other paths SHALL use Anthropic format.** **When the request path is a billing endpoint (`/v1/messages` or `/v1/chat/completions`) and the upstream returns HTTP 200, the proxy SHALL extract token usage and wrap streaming responses with the appropriate parser (`SseUsageParser` for Anthropic, `OpenAiSseUsageParser` for OpenAI).** **After the `is_active` check and before the `servers.is_empty()` check, the proxy SHALL extract the `User-Agent` header (normalizing absent or empty to `"(empty)"`), check it against `GroupConfig.blocked_user_agents` using exact string match, and if matched, return HTTP 403 with `permission_error` type and message "Access denied" using the path-appropriate error format.** **After receiving a response from an upstream server, if the server has retry config (`retry_status_codes`, `retry_count`, and `retry_delay_seconds` all non-null) and the response status code is in `retry_status_codes`, the proxy SHALL retry the same server up to `retry_count` times. Between each retry attempt, the proxy SHALL sleep for `retry_delay_seconds` seconds using `tokio::time::sleep`. If any retry attempt returns a status code NOT in `retry_status_codes`, the proxy SHALL use that response immediately (success or failover check). After all retry attempts are exhausted, the proxy SHALL use the final response and proceed to the normal failover check. The retry loop applies to both streaming and non-streaming paths.**

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

#### Scenario: Token usage extraction from streaming Anthropic response
- **WHEN** the request path is `/v1/messages` and the upstream returns a streaming HTTP 200 response
- **THEN** the system SHALL wrap the stream with `AnyParser::Anthropic` to extract `input_tokens` from `message_start` and `output_tokens` from `message_delta` events, forwarding all bytes unchanged to the client

#### Scenario: Token usage extraction from streaming OpenAI response
- **WHEN** the request path is `/v1/chat/completions` and the upstream returns a streaming HTTP 200 response
- **THEN** the system SHALL wrap the stream with `AnyParser::OpenAi` to extract token usage from the final usage chunk, forwarding all bytes unchanged to the client

#### Scenario: Token usage extraction from non-streaming Anthropic response
- **WHEN** the request path is `/v1/messages` and the upstream returns a non-streaming HTTP 200 response
- **THEN** the system SHALL parse the response JSON to extract `usage.input_tokens` and `usage.output_tokens` before returning the response to the client

#### Scenario: Token usage extraction from non-streaming OpenAI response
- **WHEN** the request path is `/v1/chat/completions` and the upstream returns a non-streaming HTTP 200 response
- **THEN** the system SHALL parse the response JSON to extract `usage.prompt_tokens` and `usage.completion_tokens` before returning the response to the client

#### Scenario: Error response format — Anthropic path
- **WHEN** the proxy generates an error (auth failure, rate limit, etc.) for a request to `/v1/messages`
- **THEN** the response SHALL use Anthropic format: `{"type":"error","error":{"type":"...","message":"..."}}`

#### Scenario: Error response format — OpenAI path
- **WHEN** the proxy generates an error for a request to `/v1/chat/completions`
- **THEN** the response SHALL use OpenAI format: `{"error":{"message":"...","type":"...","param":null,"code":null}}`

#### Scenario: Model not in group allowed list
- **WHEN** a group has `allowed_models: ["claude-sonnet-4-20250514", "gpt-4o"]` and the request body contains `"model": "claude-opus-4-20250514"`
- **THEN** the system SHALL return HTTP 403 with path-appropriate error format without contacting any upstream server

#### Scenario: Model allowed — pass-through to failover
- **WHEN** a group has `allowed_models: ["claude-sonnet-4-20250514"]` and the request body contains `"model": "claude-sonnet-4-20250514"`
- **THEN** the system SHALL proceed to the failover waterfall normally

#### Scenario: Empty allowed list — pass-through
- **WHEN** a group has an empty `allowed_models` list and the request body contains any model name
- **THEN** the system SHALL proceed to the failover waterfall normally (no model restriction)

#### Scenario: No model field in request — group has allowed models
- **WHEN** a group has a non-empty `allowed_models` list and the request body has no `model` field
- **THEN** the system SHALL return HTTP 403 with path-appropriate error format

#### Scenario: Key-level model restriction
- **WHEN** a sub-key has `key_allowed_models: ["claude-sonnet-4-20250514"]` and the request body contains `"model": "gpt-4o"` (which is in the group's allowed list but not the key's)
- **THEN** the system SHALL return HTTP 403 with path-appropriate error format

#### Scenario: Key with empty allowed list inherits group
- **WHEN** a sub-key has an empty `key_allowed_models` list and the group has `allowed_models: ["claude-sonnet-4-20250514", "gpt-4o"]`
- **THEN** the system SHALL allow any model in the group's allowed list

#### Scenario: Server rate-limited — skipped in waterfall
- **WHEN** a server has `max_requests=100, rate_window_seconds=60` and the Redis counter `rl:{group_id}:{server_id}` shows count >= 100
- **THEN** the system SHALL skip this server without sending a request and continue to the next server

#### Scenario: Per-server retry — status matches, retries succeed
- **WHEN** a server has `retry_status_codes=[503], retry_count=2, retry_delay_seconds=1.0` and the first attempt returns 503, the second attempt returns 200
- **THEN** the system SHALL sleep 1.0s, retry the same server, and return the 200 response to the client without failing over

#### Scenario: Per-server retry — all retries exhausted, then failover
- **WHEN** a server has `retry_status_codes=[503], retry_count=2, retry_delay_seconds=1.0` and all attempts (initial + 2 retries) return 503, and 503 is in the group's `failover_status_codes`
- **THEN** the system SHALL retry twice (sleeping 1.0s between each), then fall through to the failover check and move to the next server

#### Scenario: Per-server retry — response not in retry codes, no retry
- **WHEN** a server has `retry_status_codes=[503]` and the first attempt returns 429
- **THEN** the system SHALL NOT retry and SHALL proceed directly to the failover check for 429

#### Scenario: Per-server retry — retry config absent, no retry
- **WHEN** a server has `retry_status_codes=NULL` (retry disabled) and returns 503
- **THEN** the system SHALL NOT retry and SHALL proceed directly to the failover check

#### Scenario: Per-server retry — retry succeeds mid-loop, stops retrying
- **WHEN** a server has `retry_status_codes=[503], retry_count=3` and the second retry returns a status NOT in `retry_status_codes` (e.g., 200)
- **THEN** the system SHALL stop retrying and use that response immediately
