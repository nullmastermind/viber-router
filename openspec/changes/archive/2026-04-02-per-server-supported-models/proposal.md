## Why

When a group contains servers that only support specific models, the proxy currently has no way to skip those servers for incompatible requests — it will attempt the call and fail. Adding a per-server `supported_models` filter lets the failover chain route around model-incompatible servers automatically, improving reliability without requiring separate groups per model.

## What Changes

- New `supported_models TEXT[] NOT NULL DEFAULT '{}'` column on the `group_servers` table (migration required)
- `GroupServer`, `GroupServerDetail`, `AdminGroupServerDetail`, `AssignServer`, and `UpdateAssignment` structs gain a `supported_models: Vec<String>` field
- Admin API INSERT and UPDATE queries for group server assignments include `supported_models`
- Proxy failover loop gains a skip check: if `supported_models` is non-empty and the requested model is not in the list and not a key in `model_mappings`, the server is skipped silently
- `GroupConfig` Redis cache struct gains the new field (cache invalidation on update already exists)
- Admin UI edit-server dialog gains a multi-select chips input for `supported_models`, populated from the models table
- `GroupServerDetail` TypeScript interface in the frontend store gains `supported_models: string[]`

## Capabilities

### New Capabilities

- `per-server-model-filter`: Per-server `supported_models` list on group server assignments — controls which models a server will accept in the proxy failover chain

### Modified Capabilities

- `group-server-assignment`: Assignment now carries a `supported_models` field; GET and PUT endpoints expose it; empty list means no filtering (backward compatible)

## Impact

- Database: one new nullable-defaulted column, no data migration needed for existing rows
- Backend: `viber-router-api/src/models/group_server.rs`, `viber-router-api/src/routes/admin/group_servers.rs`, `viber-router-api/src/routes/proxy.rs`
- Frontend: `src/pages/GroupDetailPage.vue`, `src/stores/groups.ts`
- Redis cache: `GroupConfig` struct change requires cache invalidation on deploy (existing TTL handles this)
- No breaking changes to existing API consumers — new field defaults to empty array
