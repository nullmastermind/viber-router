## Context

Viber Router is an LLM API proxy that routes requests through groups of upstream servers with failover. Currently, when a proxy request fails (non-200 status) or triggers failover, nothing is recorded. The proxy handler silently continues to the next server or returns an error. Operators have no data to diagnose routing issues, detect degraded servers, or trace user-reported problems.

The system already uses PostgreSQL (sqlx) for persistent data and Redis (deadpool-redis) for caching. The proxy handler (`src/routes/proxy.rs`) processes all `/v1/*` requests with a failover waterfall pattern.

## Goals / Non-Goals

**Goals:**
- Record every proxy request that results in a non-200 final status, with full failover chain
- Async batch writes to avoid adding latency to proxy requests
- Monthly table partitioning for efficient retention-based cleanup
- Admin API and UI for querying and filtering logs
- Self-managing partition lifecycle (create/drop automatically)

**Non-Goals:**
- Logging successful (200) requests — volume too high, low diagnostic value
- SSE mid-stream error detection — only pre-stream status is captured
- External log aggregation (ELK, Datadog) — PostgreSQL is the single store
- Real-time alerting — monitoring only, no automated alerts
- Admin audit logs (who changed config) — separate concern for future

## Decisions

### D1: PostgreSQL over dedicated log store (ClickHouse, Loki)

Store logs in PostgreSQL using range partitioning by month.

**Rationale**: The system already depends on PostgreSQL. Adding a dedicated log store introduces operational complexity (another service to manage). With monthly partitioning, DROP PARTITION is O(1) vs DELETE of millions of rows. For the expected volume (only non-200 requests), PostgreSQL is sufficient.

**Alternative considered**: ClickHouse for columnar storage. Rejected — overkill for error-only logs, adds infrastructure dependency.

### D2: Async in-memory buffer with batch flush

Log entries are pushed into a bounded in-memory buffer (tokio mpsc channel). A background task flushes to PostgreSQL every 5 seconds or when 100 records accumulate, whichever comes first.

**Rationale**: Proxy latency is critical. Synchronous DB writes would add 1-5ms per request. The buffer decouples logging from request handling entirely.

**Buffer overflow policy**: When the channel is full (capacity: 10,000), new entries are dropped (oldest-first semantics via bounded channel). A tracing::warn is emitted. This ensures the proxy never blocks on logging.

**DB flush failure policy**: If batch INSERT fails, the batch is dropped and a tracing::warn is emitted. No retry — the next flush will attempt fresh entries. Logging must never compromise proxy availability.

### D3: Monthly range partitioning with app-managed lifecycle

The `proxy_logs` table is created as a partitioned table (RANGE on `created_at`). The application manages partitions:

- **On startup**: Ensure current month and next month partitions exist (CREATE IF NOT EXISTS)
- **Daily background job**: Create next month's partition if missing, drop partitions older than `LOG_RETENTION_DAYS`

**Rationale**: App-managed is simpler than pg_partman (no extension to install). Monthly granularity matches the 30-day default retention — typically drop 1 partition per cleanup cycle. Daily job ensures partitions are always ready.

**Partition naming**: `proxy_logs_YYYY_MM` (e.g., `proxy_logs_2026_03`)

### D4: Denormalized log schema

Log records include denormalized fields (`group_api_key`, `server_name`) to avoid JOINs on the hot query path.

**Rationale**: Logs are append-only and queried frequently with filters. JOINing against `groups` and `servers` tables on every log query would be slow. Denormalization is acceptable because log data is historical — if a server is renamed, old logs retain the name at the time of the event.

### D5: Failover chain as JSONB column

The full failover path is stored as a JSONB array in `failover_chain`:
```json
[
  {"server_id": "uuid", "server_name": "srv-1", "status": 429, "latency_ms": 120},
  {"server_id": "uuid", "server_name": "srv-2", "status": 200, "latency_ms": 340}
]
```

**Rationale**: The failover chain is variable-length and always read as a whole unit. JSONB is ideal — no separate table needed, queryable with PostgreSQL JSON operators if needed.

### D6: Log buffer via tokio::sync::mpsc bounded channel

Use `tokio::sync::mpsc::channel(10_000)` for the log buffer. The proxy handler sends log entries via the sender (non-blocking `try_send`). A dedicated tokio task receives and batches entries.

**Rationale**: mpsc channel is lock-free for the sender side, which is the hot path (proxy handler). `try_send` returns immediately — if the channel is full, the entry is dropped. The receiver task owns the flush logic (timer + batch size threshold).

### D7: Admin log query API with cursor-based pagination

The log query endpoint uses keyset/cursor pagination (WHERE created_at < $cursor ORDER BY created_at DESC LIMIT $page_size) rather than OFFSET-based pagination.

**Rationale**: OFFSET pagination degrades on large tables (must scan and skip rows). Cursor pagination is O(1) with an index on created_at. For a log table that could have millions of rows, this is essential.

**Note**: The frontend will use Quasar QTable with server-side pagination, translating page numbers to cursor values.

## Risks / Trade-offs

- **[Log loss on crash]** → Accepted trade-off. Buffer is in-memory; process crash loses unflushed entries. For error logs (not financial data), this is acceptable. Flush interval of 5s limits max loss.
- **[Partition creation race]** → Multiple app instances could try to create the same partition simultaneously. Mitigated by using `CREATE TABLE IF NOT EXISTS` which is safe for concurrent execution.
- **[Large failover_chain JSONB]** → If a group has many servers, the JSONB could be large. Mitigated by the fact that groups typically have 2-5 servers.
- **[Schema denormalization staleness]** → server_name in logs won't update if server is renamed. Accepted — logs are historical records.
