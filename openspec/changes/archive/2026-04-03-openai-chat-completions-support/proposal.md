## Why

The proxy currently meters and bills only Anthropic `/v1/messages` traffic. Customers also route OpenAI-compatible requests through the proxy, but those requests bypass subscription enforcement, usage tracking, and cost accounting entirely. Adding first-class support for `/v1/chat/completions` closes this gap without requiring any new infrastructure.

## What Changes

- Add `OpenAiSseUsageParser` to `sse_usage_parser.rs` that extracts token counts from OpenAI streaming SSE events
- Add endpoint-detection helpers (`is_billing_endpoint`, `is_openai_endpoint`) to `proxy.rs`
- Add `openai_error()` and a unified `api_error()` dispatcher so error responses match the calling API's format
- Extend subscription budget check and Telegram alerts to cover `/v1/chat/completions` in addition to `/v1/messages`
- Add OpenAI system prompt merge: inject server system prompt into `messages[0]` (role `system`) instead of the top-level `system` field
- Extract token usage from non-streaming `/v1/chat/completions` 200 responses (`prompt_tokens`, `completion_tokens`, `prompt_tokens_details.cached_tokens`)
- Use `OpenAiSseUsageParser` in the SSE streaming path for `/v1/chat/completions`
- Make `UsageTrackingStream` generic over parser type (or introduce a trait) so both parsers share the same stream wrapper

## Capabilities

### New Capabilities

- `openai-token-usage-extraction`: Extract token usage from OpenAI non-streaming and streaming responses and record it in `token_usage_logs`
- `openai-subscription-enforcement`: Apply subscription budget checks and RPM limits to `/v1/chat/completions` requests from sub-keys
- `openai-system-prompt-merge`: Inject server-configured system prompts into OpenAI request bodies via the `messages` array

### Modified Capabilities

- `token-usage-extraction`: Existing extraction logic is Anthropic-only; now must also handle OpenAI field names (`prompt_tokens`, `completion_tokens`, `prompt_tokens_details.cached_tokens`)
- `subscription-enforcement`: Subscription check currently gates on `request_path == "/v1/messages"`; must extend to `is_billing_endpoint(path)`
- `server-system-prompt`: System prompt merge currently targets Anthropic's top-level `system` field; must also handle OpenAI's `messages` array format
- `proxy-engine`: Error responses are always Anthropic-format; must dispatch to the correct format based on the request path

## Impact

- `viber-router-api/src/sse_usage_parser.rs`: new `OpenAiSseUsageParser` struct and unit tests
- `viber-router-api/src/routes/proxy.rs`: helper functions, subscription check, system prompt merge, non-streaming usage extraction, streaming usage tracking — all extended for OpenAI
- No database schema changes (existing `token_usage_logs` columns cover OpenAI fields)
- No frontend changes (usage data is already displayed generically)
- No new Cargo dependencies
