## 1. Database Migration

- [x] 1.1 Create migration `012_token_usage_logs.sql` — partitioned `token_usage_logs` table with columns: `id UUID NOT NULL DEFAULT gen_random_uuid()`, `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`, `group_id UUID NOT NULL`, `server_id UUID NOT NULL`, `model TEXT`, `input_tokens INTEGER NOT NULL`, `output_tokens INTEGER NOT NULL`, `cache_creation_tokens INTEGER`, `cache_read_tokens INTEGER`, `is_dynamic_key BOOLEAN NOT NULL DEFAULT false`, `key_hash TEXT`, `PRIMARY KEY (id, created_at)`, `PARTITION BY RANGE (created_at)`. Add indexes on `(group_id, created_at)` and `(key_hash, created_at)` ← (verify: migration runs without errors, table and indexes exist)

## 2. Usage Buffer Module

- [x] 2.1 Add `sha2` crate to `viber-router-api/Cargo.toml`
- [x] 2.2 Create `viber-router-api/src/usage_buffer.rs` — define `TokenUsageEntry` struct with fields matching the table columns. Implement `flush_task(rx: mpsc::Receiver<TokenUsageEntry>, pool: PgPool)` using the same `tokio::select!` loop pattern as `ttft_buffer.rs`: batch size 100, flush interval 5s, UNNEST batch INSERT
- [x] 2.3 Add unit tests for usage buffer — buffer overflow drops entry, send succeeds when not full, channel close signals receiver ← (verify: `cargo test` passes, tests match ttft_buffer test pattern)

## 3. SSE Token Parser Module

- [x] 3.1 Create `viber-router-api/src/sse_usage_parser.rs` — implement `SseUsageParser` struct that maintains a `Vec<u8>` carry-buffer. Method `feed(chunk: &[u8])` appends bytes and scans for complete SSE events (delimited by `\n\n`). When a complete event contains `"message_start"`, extract `input_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens` from `message.usage`. When it contains `"message_delta"`, extract `output_tokens` from `usage`. Method `finish() -> Option<ParsedUsage>` returns extracted usage only if both `input_tokens` and `output_tokens` are present
- [x] 3.2 Add unit tests for SSE parser — complete event in single chunk, event split across two chunks, message_start with cache tokens, message_delta with output_tokens, missing message_delta returns None, non-usage events are ignored, multiple content_block_delta events don't interfere ← (verify: `cargo test` passes, all 7 scenarios covered)

## 4. Key Hashing Utility

- [x] 4.1 Add a `hash_key(key: &str) -> String` function in `usage_buffer.rs` (or `sse_usage_parser.rs`) that returns the first 16 hex characters of SHA-256 of the input key ← (verify: deterministic output, 16 chars hex)

## 5. Proxy Integration — Stream Wrapping

- [x] 5.1 In `proxy.rs`, add a helper function `wrap_stream_with_usage_tracking(stream, state, group_id, server_id, model, is_dynamic_key, key_hash)` that uses `StreamExt::scan` to wrap the byte stream. The scan state holds an `SseUsageParser`. Each chunk is fed to the parser and passed through unchanged. When the stream ends, emit a `TokenUsageEntry` via `state.usage_tx.try_send`
- [x] 5.2 In the SSE response paths (both TTFT-timeout and no-timeout branches), when `request_path == "/v1/messages"` and `status == 200`: replace `Body::from_stream(first.chain(rest))` with `Body::from_stream(wrap_stream_with_usage_tracking(first.chain(rest), ...))`. Pass `is_dynamic_key` and `key_hash` computed from `parsed.dynamic_keys.get(&server.short_id)` and `server.api_key`
- [x] 5.3 In the non-SSE response path, when `request_path == "/v1/messages"` and `status == 200`: before calling `build_response`, read the response bytes, parse JSON to extract `usage.input_tokens`, `usage.output_tokens`, `usage.cache_creation_input_tokens`, `usage.cache_read_input_tokens`. If both input and output are present, emit a `TokenUsageEntry`. Then construct the response from the already-read bytes ← (verify: streaming responses still stream correctly, non-streaming responses return correct body, usage entries are emitted for both paths)

## 6. AppState and Main Integration

- [x] 6.1 Add `usage_tx: mpsc::Sender<TokenUsageEntry>` to `AppState` in `viber-router-api/src/routes/mod.rs`. Add import for `usage_buffer::TokenUsageEntry`
- [x] 6.2 In `main.rs`: create `(usage_tx, usage_rx) = tokio::sync::mpsc::channel(10_000)`, add `usage_tx` to `AppState`, spawn `usage_buffer::flush_task(usage_rx, db_pool.clone())`, join the handle on shutdown
- [x] 6.3 In `main.rs`: add `partition::ensure_partitions(&db_pool, "token_usage_logs").await` at startup and `partition::ensure_partitions` + `partition::drop_expired_partitions` for `"token_usage_logs"` in the daily maintenance loop
- [x] 6.4 Register `usage_buffer` module in `main.rs` (`mod usage_buffer;`) and `sse_usage_parser` module (`mod sse_usage_parser;`) ← (verify: `cargo check` passes, no unused import warnings)

## 7. Admin API Endpoint

- [x] 7.1 Create `viber-router-api/src/routes/admin/token_usage.rs` — implement `GET /` handler following the `ttft.rs` pattern. Accept query params: `group_id` (required UUID), `start`/`end` (optional ISO datetime), `period` (optional, default "24h"), `is_dynamic_key` (optional bool), `key_hash` (optional string). Return JSON with `servers` array containing: `server_id`, `server_name`, `model`, `total_input_tokens` (SUM), `total_output_tokens` (SUM), `total_cache_creation_tokens` (SUM), `total_cache_read_tokens` (SUM), `request_count` (COUNT). GROUP BY `server_id, server_name, model`
- [x] 7.2 Register the module in `viber-router-api/src/routes/admin/mod.rs` — add `pub mod token_usage;` and `.nest("/token-usage", token_usage::router())` in the protected router ← (verify: `cargo check` passes, endpoint is accessible behind admin auth)

## 8. Frontend — Store

- [x] 8.1 Add `TokenUsageStats` and `ServerTokenUsage` TypeScript interfaces to `src/stores/groups.ts`. Add `fetchTokenUsageStats(groupId, params)` function following the `fetchTtftStats` pattern — call `GET /api/admin/token-usage` with query params

## 9. Frontend — Group Detail Page

- [x] 9.1 Add a "Token Usage" card section to `GroupDetailPage.vue` below the existing TTFT section. Include: date range picker (using `q-date` inside `q-popup-proxy` pattern from LogsPage), server filter dropdown (`q-select` populated from group's servers), dynamic key toggle (`q-toggle` for `is_dynamic_key`), key hash filter (`q-input`)
- [x] 9.2 Add a `q-table` displaying aggregated token usage data with columns: Server, Model, Input Tokens, Output Tokens, Cache Creation, Cache Read, Requests. Format token numbers with locale separators
- [x] 9.3 Implement loading state (q-spinner), error state (q-banner with retry button), and empty state (q-banner with "No data" message) ← (verify: all 4 states render correctly, filters trigger data reload, table shows correct aggregated data)

## 10. Validate

- [x] 10.1 Run `just check` — fix all type errors and lint errors in both frontend and backend ← (verify: `just check` exits 0)
- [x] 10.2 Run `cargo test` — ensure all new unit tests pass ← (verify: all tests pass including SSE parser and buffer tests)
