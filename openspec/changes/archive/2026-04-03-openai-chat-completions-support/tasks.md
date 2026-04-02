## 1. OpenAI SSE Usage Parser

- [x] 1.1 Add `OpenAiSseUsageParser` struct to `sse_usage_parser.rs` with `buffer`, `input_tokens`, `output_tokens`, `cache_read_tokens` fields
- [x] 1.2 Implement `feed(&mut self, chunk: &[u8])` using the same `find_double_newline` buffer-splitting logic as `SseUsageParser`
- [x] 1.3 Implement `parse_data` to skip `[DONE]`, parse JSON, and extract `usage.prompt_tokens`, `usage.completion_tokens`, `usage.prompt_tokens_details.cached_tokens`
- [x] 1.4 Implement `finish(self) -> Option<ParsedUsage>` returning `None` when either token count is absent; `cache_creation_tokens` always `None`
- [x] 1.5 Add `AnyParser` enum with `Anthropic(SseUsageParser)` and `OpenAi(OpenAiSseUsageParser)` variants, implementing `feed` and `finish` by delegation
- [x] 1.6 Write unit tests: usage chunk with prompt+completion tokens, cached tokens, `[DONE]` sentinel ignored, no usage chunk returns `None`, usage split across byte chunks ← (verify: all tests pass with `cargo test -p viber-router-api sse_usage_parser`)

## 2. Proxy Helper Functions

- [x] 2.1 Add `fn is_billing_endpoint(path: &str) -> bool` returning `true` for `/v1/messages` and `/v1/chat/completions`
- [x] 2.2 Add `fn is_openai_endpoint(path: &str) -> bool` returning `true` for paths starting with `/v1/chat/`
- [x] 2.3 Add `fn openai_error(status: StatusCode, error_type: &str, message: &str) -> Response` returning `{"error":{"message":...,"type":...,"param":null,"code":null}}`
- [x] 2.4 Add `fn api_error(path: &str, status: StatusCode, error_type: &str, message: &str) -> Response` dispatching to `openai_error` or `anthropic_error` based on `is_openai_endpoint(path)`
- [x] 2.5 Replace all `anthropic_error(...)` call sites in `proxy_handler` with `api_error(&request_path, ...)` ← (verify: `cargo clippy -- -D warnings` passes; grep confirms no bare `anthropic_error` calls remain in `proxy_handler`)

## 3. Subscription Check and Telegram Alerts

- [x] 3.1 Change the subscription budget check guard from `request_path == "/v1/messages"` to `is_billing_endpoint(&request_path)` (line ~568)
- [x] 3.2 Change the Telegram alert guard at the end of `proxy_handler` (line ~1493) from `request_path == "/v1/messages"` to `is_billing_endpoint(&request_path)`
- [x] 3.3 Change the Telegram alert guard in the 400-error path (line ~1030) from the implicit `/v1/messages`-only block to use `is_billing_endpoint` ← (verify: subscription check fires for `/v1/chat/completions` sub-key requests; Telegram alert spawned for both paths on failure)

## 4. System Prompt Merge for OpenAI

- [x] 4.1 Add `fn merge_openai_system_prompt(body: &mut Value, server_prompt: &str)` that prepends or appends to `messages[0]` with role `system`
- [x] 4.2 In `transform_request_body`, add an `else if is_openai_endpoint(path) && server_system_prompt.is_some()` branch calling `merge_openai_system_prompt`
- [x] 4.3 Update `transform_request_body` signature to accept `request_path: &str` instead of `is_messages_endpoint: bool`; update all call sites ← (verify: `cargo check` passes; system prompt appears in forwarded OpenAI body when server has one configured)

## 5. Non-Streaming OpenAI Usage Extraction

- [x] 5.1 In the non-streaming 200 response path, change the guard from `request_path == "/v1/messages"` to `is_billing_endpoint(&request_path)`
- [x] 5.2 Add an `if is_openai_endpoint(&request_path)` branch inside the usage extraction block that reads `usage.prompt_tokens`, `usage.completion_tokens`, `usage.prompt_tokens_details.cached_tokens`; set `cache_creation` to `None`
- [x] 5.3 Keep the existing Anthropic branch reading `usage.input_tokens`, `usage.output_tokens`, `usage.cache_creation_input_tokens`, `usage.cache_read_input_tokens` ← (verify: `TokenUsageEntry` emitted with correct field mapping for a mock OpenAI 200 response; Anthropic path unchanged)

## 6. Streaming OpenAI Usage Tracking

- [x] 6.1 Update `UsageTrackingStream` to store `Option<AnyParser>` instead of `Option<SseUsageParser>`
- [x] 6.2 Update `wrap_stream_with_usage_tracking` to accept `parser: AnyParser` and pass it into `UsageTrackingStream`
- [x] 6.3 In both SSE streaming paths (TTFT timeout branch and no-timeout branch), change the `if request_path == "/v1/messages"` guard to `if is_billing_endpoint(&request_path)` and pass `AnyParser::Anthropic(SseUsageParser::new())` for Anthropic paths and `AnyParser::OpenAi(OpenAiSseUsageParser::new())` for OpenAI paths ← (verify: `cargo check` passes; streaming usage tracked for `/v1/chat/completions` with usage chunk present)

## 7. Final Check

- [x] 7.1 Run `just check` and fix all lint and type errors reported for both frontend and backend ← (verify: `just check` exits 0 with no warnings or errors)
