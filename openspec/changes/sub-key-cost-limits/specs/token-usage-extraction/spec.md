## MODIFIED Requirements

### Requirement: Extract token usage from non-streaming responses
When the proxy receives a non-streaming `/v1/messages` HTTP 200 response, the system SHALL extract token usage and calculate cost using cached model pricing and server rate multipliers. The cost and selected subscription_id SHALL be included in the `TokenUsageEntry` sent to the usage buffer.

#### Scenario: Cost calculated with pricing available
- **WHEN** the proxy extracts token usage and model pricing exists in the cache
- **THEN** the `TokenUsageEntry` SHALL include `cost_usd` calculated as `(input_tokens × input_1m_usd × rate_input + output_tokens × output_1m_usd × rate_output + cache_creation_tokens × cache_write_1m_usd × rate_cache_write + cache_read_tokens × cache_read_1m_usd × rate_cache_read) / 1,000,000`

#### Scenario: Cost not calculated without pricing
- **WHEN** the proxy extracts token usage but model pricing is not found in the cache
- **THEN** the `TokenUsageEntry` SHALL have `cost_usd: None` and `subscription_id: None`

### Requirement: Extract token usage from streaming SSE responses
When the proxy forwards a streaming `/v1/messages` response with HTTP 200, the system SHALL extract token usage from SSE events without modifying the forwarded stream. After the stream completes, the system SHALL calculate cost and update Redis subscription counters.

#### Scenario: Streaming cost calculation
- **WHEN** the streaming response completes and token usage has been extracted
- **THEN** the system SHALL calculate cost using cached model pricing and server rates, update Redis counters for the charged subscription, and send a `TokenUsageEntry` with `cost_usd` and `subscription_id`

#### Scenario: Streaming subscription update
- **WHEN** the streaming response completes and a subscription was selected pre-request
- **THEN** the system SHALL INCRBYFLOAT the appropriate Redis cost counters (total and per-model) for that subscription
