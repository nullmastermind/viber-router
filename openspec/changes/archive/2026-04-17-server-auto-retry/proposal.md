## Why

When a server in a group returns a transient error (e.g., 503 overload, 429 rate limit), the proxy immediately fails over to the next server. This wastes the failover slot for errors that would resolve with a brief retry on the same server, and can cause unnecessary load distribution churn.

## What Changes

- Add three optional columns to `group_servers`: `retry_status_codes`, `retry_count`, `retry_delay_seconds`
- Each server can independently declare which HTTP status codes should trigger a retry (separate from group-level `failover_status_codes`)
- The proxy loop retries the same server up to `retry_count` times with `retry_delay_seconds` delay before falling through to failover
- Admin UI adds a per-server retry config dialog (icon button on server row) with a badge indicator when configured
- Validation enforces all-or-nothing config (all three fields null or all three set), `retry_count >= 1`, `retry_delay_seconds > 0`, status codes in 400–599

## Capabilities

### New Capabilities

- `server-auto-retry`: Per-server retry configuration — status codes, count, and delay — with proxy-level retry loop before failover

### Modified Capabilities

- `group-server-assignment`: Admin API for updating a server assignment now accepts three new optional retry fields
- `proxy-engine`: Proxy request loop now performs per-server retries before triggering failover

## Impact

- Database: `group_servers` table gains 3 nullable columns (migration 040)
- Backend models: `GroupServer`, `GroupServerDetail`, `AdminGroupServerDetail`, `UpdateAssignment` structs updated
- Backend routes: `admin/group_servers.rs` update handler, `proxy.rs` server SELECT query and failover logic
- Cache: `GroupServerDetail` is cached in Redis — new fields are automatically included once the struct is updated
- Frontend: `src/stores/groups.ts` interface and `src/pages/GroupDetailPage.vue` UI
