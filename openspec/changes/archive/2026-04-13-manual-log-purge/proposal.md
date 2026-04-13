## Why

The proxy_logs table grows unboundedly as partitions accumulate over time. Admins need a way to reclaim disk space by purging old log partitions without manual SQL intervention.

## What Changes

- New GET /api/admin/logs/purge-preview?keep_days=N endpoint returns a count of rows that would be deleted
- New POST /api/admin/logs/purge endpoint drops old partitions and deletes rows from partial partitions older than keep_days
- Partition helper functions in partition.rs made pub for reuse by the new log routes
- New "Database Maintenance" section in SettingsPage with keep-days dropdown, preview count display, confirmation dialog, and purge button

## Capabilities

### New Capabilities

- `log-purge-api`: Backend endpoints for previewing and executing proxy_logs purge by partition age
- `log-purge-ui`: Admin settings UI for triggering log purge with configurable retention window and row-count preview

## Impact

- Backend: new endpoints in `viber-router-api/src/routes/admin/logs.rs`, `viber-router-api/src/partition.rs` (pub visibility)
- Frontend: `src/pages/SettingsPage.vue` (new Database Maintenance section)
- No schema migrations required — operates on existing partitioned proxy_logs table
- No breaking changes to existing API contracts
