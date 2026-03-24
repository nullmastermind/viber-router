## 1. Database & Config

- [x] 1.1 Add `LOG_RETENTION_DAYS` to `Config` struct in `src/config.rs` — optional env var, default 30, parsed as u32
- [x] 1.2 Create migration `004_create_proxy_logs` — partitioned table `proxy_logs` (RANGE on created_at) with columns: id (UUID PK), created_at (TIMESTAMPTZ), group_id (UUID), group_api_key (TEXT), server_id (UUID), server_name (TEXT), request_path (TEXT), request_method (TEXT), status_code (SMALLINT), error_type (TEXT), latency_ms (INTEGER), failover_chain (JSONB), request_model (TEXT nullable). Indexes on created_at, group_id, status_code, group_api_key ← (verify: parent table created as partitioned, indexes exist, INSERT fails without a partition present — confirming partitioning works)

## 2. Partition Manager

- [x] 2.1 Create `src/partition.rs` — functions: `ensure_partitions(pool, current_month, next_month)` creates monthly partitions using `CREATE TABLE IF NOT EXISTS proxy_logs_YYYY_MM PARTITION OF proxy_logs FOR VALUES FROM ('YYYY-MM-01') TO ('YYYY-MM+1-01')`; `drop_expired_partitions(pool, retention_days)` lists and drops partitions older than retention period
- [x] 2.2 Integrate partition management into `main.rs` — call `ensure_partitions` after migrations run (before binding listener); spawn a tokio background task that runs `ensure_partitions` + `drop_expired_partitions` once every 24 hours ← (verify: app starts clean on empty DB with partitions auto-created, daily job creates next month and drops expired)

## 3. Log Buffer

- [x] 3.1 Create `src/log_buffer.rs` — define `ProxyLogEntry` struct matching the proxy_logs schema; create `LogBuffer` with `tokio::sync::mpsc::channel(10_000)`; implement `send(entry)` using `try_send` (drop + warn on full); implement flush task that collects entries and does batch INSERT every 5 seconds or 100 records
- [x] 3.2 Add `LogBuffer` sender to `AppState` — add `log_tx: mpsc::Sender<ProxyLogEntry>` field; spawn the flush receiver task in `main.rs` before binding listener ← (verify: buffer accepts entries without blocking, flush writes to DB, overflow drops with warning, DB failure drops batch with warning)

## 4. Proxy Handler Instrumentation

- [x] 4.1 Extract request_model from request body before proxy loop — parse JSON body, read "model" field if present, store as `Option<String>`
- [x] 4.2 Instrument failover loop to build failover_chain — for each server attempt, record server_id, server_name, status (0 for connection error), and per-server latency_ms (TTFB); collect into Vec
- [x] 4.3 After failover loop completes, emit log entry if final status != 200 — construct `ProxyLogEntry` with group info, final server, total latency, failover_chain, error_type; send to `LogBuffer` via `log_tx` ← (verify: non-200 requests produce log entries with correct failover chain, 200 requests produce no log, connection errors recorded with status=0)

## 5. Admin Log API

- [x] 5.1 Create `src/routes/admin/logs.rs` — GET /api/admin/logs handler with query params: status_code (optional), group_id (optional), server_id (optional), from (optional datetime), to (optional datetime), api_key (optional text), cursor (optional datetime), page_size (optional, default 20). Build dynamic SQL query with filters, ORDER BY created_at DESC, cursor-based pagination
- [x] 5.2 Create GET /api/admin/logs/stats handler — returns total count and per-status-code counts for the filter criteria (default last 24 hours)
- [x] 5.3 Register log routes in admin router — mount at /api/admin/logs, protected by admin_auth middleware ← (verify: all filter combinations work, cursor pagination returns correct pages, stats endpoint returns accurate counts, unauthenticated requests rejected)

## 6. Admin UI — Log Viewer Page

- [x] 6.1 Create `src/pages/LogsPage.vue` — page component with QTable (server-side pagination), filter bar (status code dropdown, group select, server select, date range pickers, API key search input), loading/empty/error states
- [x] 6.2 Implement filter logic — each filter change triggers API call with updated params; debounce API key search input; reset pagination on filter change
- [x] 6.3 Implement expandable row detail — click row to expand and show failover chain visualization (server → status → server → status flow), full API key, request path, model info
- [x] 6.4 Add Logs page to router and navigation — add route in `src/router/routes.ts`, add nav item in `MainLayout.vue` left drawer ← (verify: page loads with data, all filters work, pagination navigates correctly, expandable rows show failover chain, empty/error states display correctly, nav link works)

## 7. Tests

- [x] 7.1 Unit tests for log buffer — test flush on batch size (100 entries triggers flush), flush on timer (5s), buffer overflow (try_send fails at capacity, warning emitted), empty buffer skip
- [x] 7.2 Unit tests for partition manager — test partition name generation, date range calculation for retention
- [x] 7.3 Integration test for log write path — create partition, insert batch of log entries, query back and verify data integrity ← (verify: all tests pass, buffer edge cases covered, partition lifecycle tested)
