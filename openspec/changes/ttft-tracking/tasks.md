## 1. Database Migration

- [x] 1.1 Create `007_ttft.sql` migration: add `ttft_timeout_ms INTEGER` nullable column to `groups` table, create `ttft_logs` partitioned table (id, created_at, group_id, server_id, request_model, ttft_ms, timed_out, request_path) with indexes on (group_id, server_id, created_at) ← (verify: migration runs cleanly, ttft_logs is partitioned by range on created_at, groups.ttft_timeout_ms defaults to NULL)

## 2. Backend Models

- [x] 2.1 Add `ttft_timeout_ms: Option<i32>` to `Group` and `GroupListItem` structs in `models/group.rs`
- [x] 2.2 Add `ttft_timeout_ms: Option<i32>` to `UpdateGroup` struct, update `update_group` handler in `routes/admin/groups.rs` to include the new field in the UPDATE query
- [x] 2.3 Add `ttft_timeout_ms: Option<i32>` to `GroupConfig` in `models/group_server.rs`, update `resolve_group_config` in `proxy.rs` to populate it from the DB query ← (verify: GroupConfig serializes/deserializes with new field, Redis cache invalidation works — old cached entries without field fall through to DB)

## 3. Partition Generalization

- [x] 3.1 Refactor `partition.rs`: parameterize `ensure_partitions`, `create_partition`, `drop_expired_partitions`, and `parse_partition_end_date` to accept a table name prefix instead of hardcoding `proxy_logs`
- [x] 3.2 Update `main.rs` to call partition functions for both `proxy_logs` and `ttft_logs` ← (verify: both tables get current+next month partitions on startup, existing proxy_logs partition tests still pass)

## 4. TTFT Buffer

- [x] 4.1 Create `ttft_buffer.rs` with `TtftLogEntry` struct (group_id, server_id, request_model, ttft_ms Option, timed_out, request_path, created_at) and `flush_task` function following the same mpsc batch-flush pattern as `log_buffer.rs`
- [x] 4.2 Add `ttft_tx: mpsc::Sender<TtftLogEntry>` to `AppState` in `routes/mod.rs`
- [x] 4.3 Create the mpsc channel and spawn the TTFT flush task in `main.rs` ← (verify: channel created with 10_000 capacity, flush task spawned, AppState includes ttft_tx)

## 5. TTFT Measurement & Auto-Switch in Proxy

- [x] 5.1 In `proxy_handler`: after `send().await` returns 200, detect SSE via content-type header (before calling `build_response`). When TTFT is enabled + multiple servers + not last server: use `tokio::time::timeout` on `stream.next()` to peek the first chunk
- [x] 5.2 On timeout: record timed_out TTFT entry via `ttft_tx`, drop the stream (closes connection), continue failover loop to next server
- [x] 5.3 On success (first chunk received): record TTFT entry via `ttft_tx`, build SSE response by chaining the first chunk with the rest of the stream using `futures_util::stream::once` + `stream`
- [x] 5.4 On empty stream (stream.next() returns None): treat as connection error, continue to next server
- [x] 5.5 When TTFT is disabled, single server, or last server: always measure and record TTFT (no timeout), then call existing `build_response` unchanged ← (verify: non-SSE requests unchanged, SSE with TTFT disabled uses existing path, SSE with TTFT enabled on non-last server applies timeout, last server always waits indefinitely, TTFT recorded for all SSE requests regardless of timeout config)

## 6. TTFT Stats API

- [x] 6.1 Create `routes/admin/ttft.rs` with `GET /` handler: accepts `group_id` (required UUID) and `period` (optional, default "1h") query params, returns per-server aggregated stats (avg, p50, p95, timeout_count, total_count) and individual data points
- [x] 6.2 Register the route in `routes/admin/mod.rs`: add `pub mod ttft;` and `.nest("/ttft-stats", ttft::router())` inside the protected router ← (verify: endpoint returns correct stats shape, requires admin auth, returns 400 on missing group_id, returns empty servers array when no data)

## 7. Frontend Dependencies & Store

- [x] 7.1 Install `chart.js` and `vue-chartjs`: `bun add chart.js vue-chartjs`
- [x] 7.2 Add `ttft_timeout_ms` field to group types in `stores/groups.ts`, update `updateGroup` input type to include `ttft_timeout_ms?: number | null`
- [x] 7.3 Add `fetchTtftStats(groupId: string)` action to groups store that calls `GET /api/admin/ttft-stats?group_id=<id>&period=1h` ← (verify: types match API response shape)

## 8. Frontend UI

- [x] 8.1 Add TTFT timeout input field to the Properties card in `GroupDetailPage.vue` (number input, empty = null/disabled)
- [x] 8.2 Create TTFT chart card in `GroupDetailPage.vue` below the Servers card: Line chart with Chart.js showing per-server TTFT over last hour, timeout points marked distinctly, empty state when no data
- [x] 8.3 Add 30-second auto-refresh interval for the TTFT chart data (clear on unmount) ← (verify: chart renders with real data, timeout points visually distinct, auto-refresh works, empty state shown when no data, ttft_timeout_ms saves correctly from Properties card)

## 9. Tests

- [x] 9.1 Unit tests for partition.rs generalization: verify parameterized functions work for both `proxy_logs` and `ttft_logs` prefixes
- [x] 9.2 Unit tests for TTFT buffer: channel send/drop behavior (same pattern as existing log_buffer tests)
- [x] 9.3 Integration test for proxy TTFT flow — deferred (requires running server with mock upstream, manual testing)

## 10. Final Check

- [x] 10.1 Run `just check` — fix all type errors, lint errors, and clippy warnings ← (verify: `just check` passes cleanly)
