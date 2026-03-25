## Context

Viber Router proxies Anthropic API requests through a failover waterfall of upstream servers. Each group has an ordered list of servers; the proxy tries them in priority order until one succeeds. The proxy currently logs request metadata (`proxy_logs`) and time-to-first-token measurements (`ttft_logs`), both using a buffered mpsc channel → batch INSERT pattern into partitioned PostgreSQL tables.

There is no token usage tracking. Operators cannot see how many input/output tokens each server or key consumes. The Anthropic Messages API returns token usage in both streaming (SSE) and non-streaming responses:
- Streaming: `input_tokens` in `message_start` event, `output_tokens` in `message_delta` event (cumulative)
- Non-streaming: `usage` object at top level of JSON response

The proxy currently forwards SSE streams as pass-through (`Body::from_stream(first.chain(rest))`). It already iterates stream chunks for TTFT measurement but does not parse SSE event content.

Dynamic keys (via `-rsv-` syntax in the API key header) allow users to inject per-server upstream keys. Tracking which requests use dynamic keys vs server default keys is needed for per-key usage attribution.

## Goals / Non-Goals

**Goals:**
- Extract input/output/cache token counts from `/v1/messages` responses (both streaming and non-streaming) without modifying the forwarded response
- Persist per-request token usage to a partitioned `token_usage_logs` table via buffered batch inserts
- Track whether each request used a dynamic key and store a hash of the resolved upstream key
- Provide an admin API endpoint for aggregated token usage queries with filters
- Display token usage stats in the Group detail admin page

**Non-Goals:**
- Tracking token usage for `/v1/messages/count_tokens` (not real usage)
- Tracking non-200 responses (error responses have no meaningful usage)
- Pre-aggregation into daily summary tables (direct query with partition pruning is sufficient)
- OpenAI-compatible endpoint token tracking (Anthropic only for now)
- Real-time streaming usage display (batch insert has ~5s flush delay)

## Decisions

### 1. Stream-peeking via custom UsageTrackingStream wrapper

**Choice**: Wrap the forwarded SSE stream in a custom `UsageTrackingStream<S>` struct that implements `futures_util::Stream`. The wrapper carries an `SseUsageParser` across chunks, extracts usage from `message_start` and `message_delta` events, and emits the usage entry when the inner stream returns `None`.

**Alternatives considered**:
- Collect entire response then forward — rejected because it destroys streaming for the client.
- `StreamExt::scan` combinator — rejected because `scan` does not provide access to accumulated state on stream termination (when the inner stream yields `None`), making it impossible to emit the final usage entry.
- Use `count_tokens` server for input counting — rejected because it adds an extra API call per request and cannot provide output tokens.

**Rationale**: The custom `Stream` impl carries the `SseUsageParser` as owned state, passes each `Bytes` chunk through unchanged via `poll_next`, and on stream end (`Poll::Ready(None)`), calls `parser.finish()` to emit the usage entry via `try_send`. Zero-copy pass-through — the client sees no difference.

**SSE buffer handling**: `bytes_stream()` produces arbitrary byte chunks, not line-aligned SSE events. The parser maintains a `Vec<u8>` carry-buffer. On each chunk, append bytes, scan for complete SSE events (terminated by `\n\n`), parse only events containing `"message_start"` or `"message_delta"`. On stream end (`None`), emit the usage entry. On stream error, drop the carry-buffer and skip emit.

### 2. Non-SSE extraction at call site, not in build_response

**Choice**: For non-streaming `/v1/messages` HTTP 200 responses, parse the JSON body to extract usage BEFORE calling `build_response`, since `build_response` is shared with non-200 paths and lacks request context.

**Rationale**: `build_response` does not have access to `request_path` or group context. The guard (`/v1/messages` + HTTP 200) must be applied in `proxy_handler`.

### 3. Separate token_usage_logs table (not columns on proxy_logs)

**Choice**: New partitioned table `token_usage_logs` with its own buffer and flush task.

**Alternatives considered**:
- Add columns to `proxy_logs` — rejected because proxy_logs only records failover/error events, not every successful request. Token usage needs to be logged for every successful `/v1/messages` response.

**Rationale**: Clean separation. Independent partitioning and retention. Query patterns are different (usage aggregation vs error investigation).

### 4. Key tracking: is_dynamic_key + key_hash

**Choice**: `is_dynamic_key: bool` indicates whether the request used a dynamic key. `key_hash: text` stores a truncated SHA-256 of the resolved upstream key (the key for the winning server: `parsed.dynamic_keys.get(&server.short_id)` if dynamic, else `server.api_key`).

**Rationale**: Never store raw API keys. The hash enables per-key grouping and filtering without exposing secrets. Truncated to first 16 hex chars for readability while maintaining uniqueness in practice.

### 5. Admin endpoint: flat route following TTFT pattern

**Choice**: `GET /api/admin/token-usage?group_id=<uuid>&start=<iso>&end=<iso>` — flat route with query params, matching the existing `/api/admin/ttft-stats` pattern.

**Alternatives considered**:
- Nested route `/api/admin/groups/{id}/token-usage` — valid but inconsistent with TTFT precedent.

### 6. SHA-256 via sha2 crate

**Choice**: Add `sha2` crate to `Cargo.toml` for key hashing.

**Rationale**: Standard, well-audited crate. No existing hashing dependency in the project.

## Risks / Trade-offs

- **[SSE chunk splitting]** SSE events may span multiple byte chunks → Mitigated by carry-buffer that accumulates bytes until `\n\n` delimiter is found.
- **[Scan combinator complexity]** Custom stream wrapper adds code complexity to proxy.rs → Mitigated by extracting the SSE parser into a separate module with unit tests.
- **[Buffer full drops]** If `usage_tx` channel is full, entries are dropped silently (same as proxy_logs/ttft_logs) → Acceptable trade-off; logged via `tracing::warn`.
- **[Partial stream]** If stream ends without `message_delta` (client disconnect, upstream error), no usage is logged → By design; incomplete requests have unreliable token counts.
- **[Query performance]** Direct aggregation queries on large tables → Mitigated by partition pruning on `created_at` and indexes on `(group_id, created_at)`.
