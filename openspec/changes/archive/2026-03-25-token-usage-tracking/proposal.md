## Why

There is no visibility into how many tokens each server/key consumes within a group. Operators cannot answer "which key used the most tokens this week?" or "what's the input/output breakdown per server?" without manually checking upstream dashboards. Token usage stats per server per key with date range filtering enables cost attribution and usage monitoring directly in the admin UI.

## What Changes

- Add a stream-peeking layer in the proxy that extracts `input_tokens` and `output_tokens` from Anthropic SSE events (`message_start` and `message_delta`) without modifying the forwarded stream.
- For non-streaming `/v1/messages` responses (HTTP 200), parse the JSON response body to extract the top-level `usage` object.
- Track cache token fields (`cache_creation_input_tokens`, `cache_read_input_tokens`) when present.
- Record whether the request used a dynamic key (via `-rsv-` syntax) and store a truncated SHA-256 hash of the resolved upstream key for per-key filtering.
- Create a new partitioned `token_usage_logs` table following the same pattern as `proxy_logs` and `ttft_logs`.
- Add a buffered writer (`usage_buffer.rs`) using the existing mpsc channel + batch insert pattern.
- Add an admin API endpoint for querying aggregated token usage (SUM grouped by server/model, with date range and key filters).
- Add a "Token Usage" section to the Group detail page with date range picker, server filter, and dynamic key filter.

## Capabilities

### New Capabilities
- `token-usage-extraction`: Extract input/output/cache token counts from Anthropic API responses (both streaming SSE and non-streaming JSON) at the proxy layer without modifying the forwarded response.
- `token-usage-storage`: Persist per-request token usage records to a partitioned PostgreSQL table via buffered batch inserts.
- `token-usage-stats-api`: Admin API endpoint for querying aggregated token usage with filters (group, server, model, date range, dynamic key).
- `token-usage-ui`: Token usage statistics section in the Group detail admin page with filters and aggregate display.

### Modified Capabilities
- `proxy-engine`: The proxy handler gains a stream-wrapping step for `/v1/messages` requests that peeks at SSE events to extract token usage, and a response-body inspection step for non-streaming 200 responses.

## Impact

- **Backend**: New module `usage_buffer.rs`, new migration for `token_usage_logs` table, new admin route module, changes to `proxy.rs` (stream wrapper + non-SSE extraction), `AppState` gains `usage_tx` sender, `main.rs` gains channel setup + partition maintenance for new table.
- **Frontend**: New section in `GroupDetailPage.vue`, new store function in `groups.ts`, new API call.
- **Dependencies**: `sha2` crate added to `Cargo.toml` for key hashing.
- **Database**: New partitioned table `token_usage_logs` with indexes on `(group_id, created_at)` and `(key_hash, created_at)`.
