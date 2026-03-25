## Why

Admins need to temporarily disable a server within a specific group without removing it entirely. Currently the only options are to delete the server assignment (losing priority and model_mappings config) or to leave it active. A disable toggle allows quick maintenance operations where a server can be taken out of the failover chain and brought back with all config preserved.

## What Changes

- Add `is_enabled` boolean column to `group_servers` table (default `true`)
- Proxy engine excludes disabled servers from the failover chain (filtered at DB query level, not included in Redis cache)
- Admin API returns `is_enabled` in GroupServerDetail; `UpdateAssignment` accepts `is_enabled`
- Frontend shows inline toggle switch per server row; disabled servers appear dimmed with strikethrough name

## Capabilities

### New Capabilities

### Modified Capabilities
- `group-server-assignment`: Add `is_enabled` field to group-server assignment. New servers default to enabled. Admins can toggle via PUT endpoint. GroupServerDetail includes `is_enabled` in response.
- `config-cache`: Proxy query filters out disabled servers (`is_enabled = false`) so they are never included in the cached GroupConfig.
- `admin-ui`: Server list in group detail page shows a toggle switch per server. Disabled servers are visually dimmed with strikethrough name.

## Impact

- **Database**: New migration adding `is_enabled` column to `group_servers`
- **Backend models**: `GroupServer`, `GroupServerDetail`, `UpdateAssignment` structs gain `is_enabled` field
- **Backend routes**: `group_servers.rs` update handler, `groups.rs` detail query, `proxy.rs` server query
- **Frontend**: `stores/groups.ts` type update, `GroupDetailPage.vue` UI toggle
- **Cache**: Redis cache invalidated on toggle (existing invalidation pattern via `invalidate_group_cache`)
