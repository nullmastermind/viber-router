## Why

Large requests with many input tokens are expensive on per-token-priced servers. Without a way to route based on token volume, the proxy has no mechanism to steer oversized requests away from cost-sensitive servers in the failover waterfall. Adding a per-server-in-group token threshold enables operators to reserve cheap/flat-rate servers for large requests while protecting expensive per-token servers from unexpectedly high costs.

## What Changes

- New nullable integer column `max_input_tokens` on the `group_servers` table
- Database migration 031 adds the column
- Proxy estimates input tokens from the request body (strip image blocks, divide filtered JSON length by 4) before entering the failover loop
- During the failover waterfall, servers with a configured `max_input_tokens` are skipped when the estimated token count exceeds the threshold
- `GroupServer`, `GroupServerDetail`, `AdminGroupServerDetail`, and `UpdateAssignment` Rust structs gain the new field
- SQL query in proxy.rs that fetches `GroupServerDetail` includes the new column
- `AssignServer` struct accepts an optional `max_input_tokens` at assignment time
- Admin API `PUT /api/admin/groups/{group_id}/servers/{server_id}` accepts `max_input_tokens: Option<Option<i32>>`
- Frontend `GroupServerDetail` TypeScript interface gains `max_input_tokens: number | null`
- Edit Server dialog in `GroupDetailPage.vue` adds a "Max Input Tokens" number input
- Server list shows a badge (e.g., "≤30K tokens") when `max_input_tokens` is set

## Capabilities

### New Capabilities

- `max-input-token-threshold`: Per-server-in-group token threshold that causes the proxy failover loop to skip servers when estimated input tokens exceed the configured limit

### Modified Capabilities

- `group-server-assignment`: The assignment record gains a new optional `max_input_tokens` field that can be set at creation and updated via the existing update endpoint
- `proxy-engine`: Failover loop now estimates input tokens and evaluates per-server token thresholds before attempting each server

## Impact

- **Database**: `viber-router-api/migrations/031_add_max_input_tokens.sql`
- **Backend models**: `viber-router-api/src/models/group_server.rs`
- **Backend proxy**: `viber-router-api/src/routes/proxy.rs` (token estimation + threshold check in failover loop)
- **Backend admin API**: `viber-router-api/src/routes/admin/group_servers.rs` (update_assignment handler)
- **Frontend store**: `src/stores/groups.ts` (interface + updateAssignment call)
- **Frontend UI**: `src/pages/GroupDetailPage.vue` (edit dialog, server list badge)
- **No breaking changes**: NULL default preserves existing behavior; all existing assignments continue to work unchanged
