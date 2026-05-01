## Purpose
TBD

## Requirements
### Requirement: OpenAI SSE usage parser
The system SHALL provide an `OpenAiSseUsageParser` struct in `sse_usage_parser.rs` that parses OpenAI streaming SSE events and returns a `ParsedUsage` value. The parser SHALL buffer incoming bytes, split on `\n\n` event boundaries, and for each `data: ` line attempt JSON parsing. The parser SHALL extract `usage.prompt_tokens` as `input_tokens` and `usage.completion_tokens` as `output_tokens` from any chunk that contains a `usage` object. The parser SHALL skip `data: [DONE]` lines without error. `cache_creation_tokens` SHALL always be `None`. `cache_read_tokens` SHALL be extracted from `usage.prompt_tokens_details.cached_tokens` when present.

#### Scenario: Usage chunk with prompt and completion tokens
- **WHEN** the parser receives `data: {"choices":[],"usage":{"prompt_tokens":27,"completion_tokens":9,"total_tokens":36}}\n\n`
- **THEN** `finish()` SHALL return `ParsedUsage { input_tokens: 27, output_tokens: 9, cache_creation_tokens: None, cache_read_tokens: None }`

#### Scenario: Usage chunk with cached tokens
- **WHEN** the parser receives a usage chunk with `"prompt_tokens_details": {"cached_tokens": 10}`
- **THEN** `finish()` SHALL return `ParsedUsage` with `cache_read_tokens: Some(10)`

#### Scenario: DONE sentinel is ignored
- **WHEN** the parser receives `data: [DONE]\n\n`
- **THEN** the parser SHALL NOT return an error and SHALL continue processing subsequent chunks

#### Scenario: No usage chunk present
- **WHEN** the stream ends without any chunk containing a `usage` object
- **THEN** `finish()` SHALL return `None`

#### Scenario: Usage split across byte chunks
- **WHEN** the SSE event bytes are delivered in multiple `feed()` calls that split the event mid-line
- **THEN** `finish()` SHALL still return the correct `ParsedUsage` after all chunks are fed

### Requirement: AnyParser enum for stream wrapper
The system SHALL provide an `AnyParser` enum in `sse_usage_parser.rs` with variants `Anthropic(SseUsageParser)` and `OpenAi(OpenAiSseUsageParser)`. `AnyParser` SHALL expose `feed(&mut self, chunk: &[u8])` and `finish(self) -> Option<ParsedUsage>` methods that delegate to the inner parser. `UsageTrackingStream` SHALL store `Option<AnyParser>` instead of `Option<SseUsageParser>`.

#### Scenario: Anthropic variant delegates correctly
- **WHEN** `AnyParser::Anthropic(parser)` receives `feed()` and `finish()` calls
- **THEN** the results SHALL be identical to calling the same methods directly on `SseUsageParser`

#### Scenario: OpenAI variant delegates correctly
- **WHEN** `AnyParser::OpenAi(parser)` receives `feed()` and `finish()` calls
- **THEN** the results SHALL be identical to calling the same methods directly on `OpenAiSseUsageParser`

### Requirement: Extract token usage from non-streaming OpenAI responses
When the proxy receives a non-streaming `/v1/chat/completions` HTTP 200 response, the system SHALL extract token usage from the response JSON and record a `TokenUsageEntry`. The system SHALL read `usage.prompt_tokens` as `input_tokens`, `usage.completion_tokens` as `output_tokens`, and `usage.prompt_tokens_details.cached_tokens` as `cache_read_tokens`. `cache_creation_tokens` SHALL be `None`.

#### Scenario: Non-streaming OpenAI response with usage
- **WHEN** the proxy receives a 200 response from `/v1/chat/completions` with `{"usage":{"prompt_tokens":100,"completion_tokens":50,"total_tokens":150}}`
- **THEN** the system SHALL emit a `TokenUsageEntry` with `input_tokens=100`, `output_tokens=50`, `cache_creation_tokens=None`, `cache_read_tokens=None`

#### Scenario: Non-streaming OpenAI response with cached tokens
- **WHEN** the response includes `"prompt_tokens_details": {"cached_tokens": 20}`
- **THEN** the `TokenUsageEntry` SHALL have `cache_read_tokens=Some(20)`

#### Scenario: Non-streaming OpenAI response missing usage
- **WHEN** the response JSON has no `usage` field
- **THEN** the system SHALL NOT emit a `TokenUsageEntry` and SHALL return the response body unchanged

### Requirement: Extract token usage from streaming OpenAI responses
When the proxy forwards a streaming `/v1/chat/completions` HTTP 200 response, the system SHALL wrap the stream with `UsageTrackingStream` using `AnyParser::OpenAi(OpenAiSseUsageParser::new())`. After the stream completes, the system SHALL calculate cost and emit a `TokenUsageEntry` using the same logic as for Anthropic streaming responses.

#### Scenario: Streaming OpenAI response with usage chunk
- **WHEN** the client sends a streaming request to `/v1/chat/completions` and the upstream includes a final usage chunk
- **THEN** after the stream ends, the system SHALL emit a `TokenUsageEntry` with the extracted token counts

#### Scenario: Streaming OpenAI response without usage chunk
- **WHEN** the client does not set `stream_options.include_usage: true` and the upstream sends no usage chunk
- **THEN** `finish()` returns `None` and no `TokenUsageEntry` is emitted (graceful no-op)
