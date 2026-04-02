## Context

The proxy currently handles two response shapes: non-streaming JSON and streaming SSE. Both paths have billing/metering logic that is hardcoded to Anthropic's `/v1/messages` endpoint. The `SseUsageParser` reads Anthropic-specific event types (`message_start`, `message_delta`). The non-streaming path reads Anthropic-specific field names (`usage.input_tokens`, `usage.output_tokens`, `usage.cache_creation_input_tokens`, `usage.cache_read_input_tokens`). Error responses always use Anthropic's `{"type":"error","error":{...}}` envelope. System prompt injection targets Anthropic's top-level `system` field.

OpenAI-compatible requests already flow through the proxy (routing, failover, circuit breaking, rate limiting all work generically), but they are invisible to the billing layer.

## Goals / Non-Goals

**Goals:**
- Meter and bill `/v1/chat/completions` requests identically to `/v1/messages`
- Return error responses in the format matching the calling API (OpenAI format for OpenAI paths)
- Inject server system prompts into OpenAI request bodies via the `messages` array
- Add `OpenAiSseUsageParser` with the same `ParsedUsage` output as `SseUsageParser`
- No new infrastructure, no schema changes, no new Cargo dependencies

**Non-Goals:**
- Translating between Anthropic and OpenAI request/response formats
- Supporting other OpenAI endpoints beyond `/v1/chat/completions` for billing (routing still works for all paths)
- Modifying the frontend

## Decisions

### Decision: Path-based dispatch, not a trait object

**Options considered:**
1. Introduce a `UsageParser` trait and make `UsageTrackingStream` generic over `T: UsageParser`
2. Use an enum `AnyParser { Anthropic(SseUsageParser), OpenAi(OpenAiSseUsageParser) }` stored in `UsageTrackingStream`
3. Duplicate `UsageTrackingStream` into two concrete types

**Choice: Option 2 (enum).**

Rationale: The stream wrapper is already a concrete struct with many fields. A trait object (`Box<dyn UsageParser>`) adds a heap allocation per request. A generic parameter works but requires `wrap_stream_with_usage_tracking` to be generic, which propagates the type parameter through the call sites. An enum keeps the call sites simple — the caller passes a `parser: AnyParser` value and the stream wrapper is unchanged. The enum is defined in `sse_usage_parser.rs` alongside both parsers.

### Decision: `is_billing_endpoint` and `is_openai_endpoint` as free functions

Both helpers are called in multiple places in `proxy.rs` (subscription check, system prompt merge, non-streaming extraction, streaming wrap, Telegram alerts). Defining them as `fn is_billing_endpoint(path: &str) -> bool` and `fn is_openai_endpoint(path: &str) -> bool` at module level keeps the call sites readable and avoids repeating string literals.

### Decision: `api_error` dispatcher replaces direct `anthropic_error` calls

Rather than adding `if is_openai_endpoint { openai_error(...) } else { anthropic_error(...) }` at every error site, a single `api_error(path, status, error_type, message)` function dispatches internally. This keeps error-site code unchanged in structure and makes future API additions a one-line change in `api_error`.

### Decision: OpenAI system prompt merge strategy

OpenAI has no top-level `system` field. The canonical way to inject a system prompt is to prepend a `{"role":"system","content":"..."}` message. If the client already sent a system message as the first element, the server prompt is appended to its `content` string (same concatenation semantics as Anthropic). This preserves client intent while injecting the server constraint.

### Decision: `cache_creation_tokens` is always `None` for OpenAI

OpenAI has no equivalent of Anthropic's prompt caching write cost. The `token_usage_logs` column accepts NULL, and `calculate_cost` already handles `None` for cache creation. No special handling needed.

### Decision: OpenAI streaming requires `stream_options.include_usage: true`

The proxy does NOT inject `stream_options: {"include_usage": true}` into outgoing OpenAI requests. If the client does not request it, the final usage chunk will not be present and `OpenAiSseUsageParser.finish()` returns `None` — the same graceful no-op as when Anthropic usage events are absent. This keeps the proxy transparent and avoids mutating client requests.

## Risks / Trade-offs

- **Missing usage data when client omits `stream_options`** → Mitigation: document that clients must set `stream_options.include_usage: true` for streaming billing to work. Non-streaming always has usage.
- **OpenAI `[DONE]` sentinel is not JSON** → `OpenAiSseUsageParser` must skip lines where `data` equals `[DONE]` before attempting JSON parse. Already handled in the design.
- **Subscription error response format** → When a sub-key on an OpenAI path hits the budget limit, the proxy now returns OpenAI-format 429 instead of Anthropic-format. This is correct behavior but is a visible change for any client that was previously hitting the limit on an OpenAI path (they would have received Anthropic format before, which was wrong anyway).

## Migration Plan

1. Deploy backend with the new code — no DB migrations required
2. No rollback complexity: if reverted, OpenAI paths simply lose metering again (same as today)

## Open Questions

None — all decisions are resolved above.
