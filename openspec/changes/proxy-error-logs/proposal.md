## Why

The viber-router proxy currently has zero observability into request failures. When an upstream server returns a non-200 status or a connection error triggers failover, nothing is recorded. Operators cannot diagnose routing issues, detect degraded upstream servers, or trace user-reported problems back to specific requests. For a system designed for maximum uptime, this is a critical gap.

## What Changes

- Add a `proxy_logs` PostgreSQL table (range-partitioned by month) to store every proxy request that results in a non-200 status code
- Implement an async in-memory log buffer that flushes to the database in batches (every 5 seconds or 100 records, whichever comes first) to avoid adding latency to proxy requests
- Add automatic partition management: the application creates monthly partitions on startup and runs a daily background job to create upcoming partitions and drop partitions older than the configurable retention period (default 30 days via `LOG_RETENTION_DAYS`)
- Instrument the proxy handler to capture: group info, server info, status code, error type, latency (TTFB), failover chain, request model, and the group's API key for tracing
- Add backend admin API endpoints for querying logs with server-side pagination and filtering (by status code, group, server, date range, API key search)
- Add an admin UI log viewer page with filters, server-side pagination, expandable failover chain detail, and full component states (loading, empty, error, data)

## Capabilities

### New Capabilities
- `proxy-error-logging`: Captures non-200 proxy request outcomes into PostgreSQL with async batch writes, including failover chain tracing
- `log-partition-management`: Automatic monthly partition creation and retention-based cleanup of the proxy_logs table
- `admin-log-viewer`: Backend API and frontend UI for browsing, filtering, and searching proxy error logs

### Modified Capabilities
- `server-setup`: Add `LOG_RETENTION_DAYS` to optional env vars (default 30), spawn log buffer flush task and partition management background job on startup

## Impact

- **Database**: New migration for `proxy_logs` partitioned table with indexes on created_at, group_id, status_code, group_api_key
- **Backend**: New modules for log buffer, partition manager, log query API; modified proxy handler to emit log entries; modified main.rs to spawn background tasks
- **Frontend**: New LogsPage component with filters and expandable rows; new route in admin section
- **Dependencies**: No new crate dependencies expected (uses existing sqlx, tokio, serde_json, chrono)
- **Config**: New optional env var `LOG_RETENTION_DAYS` (default 30)
