## Why

When an upstream server starts failing repeatedly (returning failover status codes or connection errors), the proxy keeps sending requests to it on every failover cycle, creating unnecessary load on a broken server and increasing latency for all requests. Admins must manually disable the server via the UI toggle. A circuit breaker auto-disables servers after repeated failures and re-enables them after a cooldown period.

## What Changes

- Add 3 nullable columns to `group_servers` table: `cb_max_failures`, `cb_window_seconds`, `cb_cooldown_seconds` (all-or-nothing validation)
- Track error counts per (group_id, server_id) in Redis with TTL-based sliding window
- Auto-disable servers in the proxy failover loop when error threshold is reached (skip via Redis key, not DB write)
- Auto re-enable when Redis cooldown key expires
- Send Telegram alert when circuit breaker trips and when server re-enables (check-on-next-request pattern)
- Add `GET /api/admin/groups/{id}/circuit-status` endpoint returning open circuit states with remaining seconds
- Show "Circuit Open (Xm Ys)" countdown badge on server rows in GroupDetailPage
- Add circuit breaker configuration fields (max failures, failure window, cooldown) to server edit dialog with Vietnamese descriptions

## Capabilities

### New Capabilities
- `circuit-breaker`: Circuit breaker logic — Redis-based error counting, threshold detection, auto-disable/re-enable, and Telegram alerts on state transitions

### Modified Capabilities
- `group-server-assignment`: Add `cb_max_failures`, `cb_window_seconds`, `cb_cooldown_seconds` nullable fields to group-server assignment; all-or-nothing validation
- `proxy-engine`: Skip circuit-broken servers in failover loop; increment error counter on failover/connection errors; trip breaker when threshold reached
- `telegram-alert-delivery`: Send alerts on circuit breaker trip and re-enable events

## Impact

- **Database**: New migration adding 3 columns to `group_servers`
- **Backend models**: `GroupServer`, `GroupServerDetail`, `UpdateAssignment`, `GroupConfig` structs gain circuit breaker fields
- **Backend routes**: `proxy.rs` (circuit check + error counting), `group_servers.rs` (validation), new `circuit_status` endpoint in `groups.rs`
- **Redis**: New key patterns `cb:err:{group_id}:{server_id}` and `cb:open:{group_id}:{server_id}`
- **Frontend**: `GroupDetailPage.vue` (badge + edit fields), `stores/groups.ts` (types + API call)
- **Telegram**: New alert message format for circuit breaker events
