## Purpose
TBD

## Requirements
### Requirement: Extract token usage from non-streaming responses
When the proxy receives a non-streaming HTTP 200 response on a billing endpoint (`/v1/messages` or `/v1/chat/completions`), the system SHALL extract token usage and calculate cost using cached model pricing and server rate multipliers. The cost and selected subscription_id SHALL be included in the `TokenUsageEntry` sent to the usage buffer. For `/v1/messages` responses, the system SHALL read `usage.input_tokens`, `usage.output_tokens`, `usage.cache_creation_input_tokens`, and `usage.cache_read_input_tokens`. For `/v1/chat/completions` responses, the system SHALL read `usage.prompt_tokens` as `input_tokens`, `usage.completion_tokens` as `output_tokens`, `usage.prompt_tokens_details.cached_tokens` as `cache_read_tokens`, and SHALL set `cache_creation_tokens` to `None`.

#### Scenario: Cost calculated with pricing available
- **WHEN** the proxy extracts token usage and model pricing exists in the cache
- **THEN** the `TokenUsageEntry` SHALL include `cost_usd` calculated as `(input_tokens × input_1m_usd × rate_input + output_tokens × output_1m_usd × rate_output + cache_creation_tokens × cache_write_1m_usd × rate_cache_write + cache_read_tokens × cache_read_1m_usd × rate_cache_read) / 1,000,000`

#### Scenario: Cost not calculated without pricing
- **WHEN** the proxy extracts token usage but model pricing is not found in the cache
- **THEN** the `TokenUsageEntry` SHALL have `cost_usd: None` and `subscription_id: None`

### Requirement: Extract token usage from streaming SSE responses
When the proxy forwards a streaming HTTP 200 response on a billing endpoint (`/v1/messages` or `/v1/chat/completions`), the system SHALL extract token usage from SSE events without modifying the forwarded stream. For `/v1/messages`, the system SHALL use `SseUsageParser` (Anthropic event types). For `/v1/chat/completions`, the system SHALL use `OpenAiSseUsageParser` (OpenAI usage chunk format). After the stream completes, the system SHALL calculate cost and update Redis subscription counters.

#### Scenario: Streaming cost calculation
- **WHEN** the streaming response completes and token usage has been extracted
- **THEN** the system SHALL calculate cost using cached model pricing and server rates, update Redis counters for the charged subscription, and send a `TokenUsageEntry` with `cost_usd` and `subscription_id`

#### Scenario: Streaming subscription update
- **WHEN** the streaming response completes and a subscription was selected pre-request
- **THEN** the system SHALL INCRBYFLOAT the appropriate Redis cost counters (total and per-model) for that subscription
