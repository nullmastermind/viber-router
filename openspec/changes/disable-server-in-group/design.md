## Context

Viber Router uses a `group_servers` join table to associate servers with groups, each with a priority and model_mappings. The proxy engine queries this table (or reads from Redis cache) to build the failover chain. Currently there is no way to temporarily disable a server within a group — the only option is to remove the assignment entirely, which loses configuration.

## Goals / Non-Goals

**Goals:**
- Allow admins to enable/disable individual server assignments within a group
- Disabled servers are excluded from the proxy failover chain
- Preserve all configuration (priority, model_mappings) when a server is disabled
- Provide instant inline toggle in the admin UI

**Non-Goals:**
- Global server enable/disable (server-level, affecting all groups) — out of scope
- Bulk enable/disable of servers across groups — out of scope
- Historical tracking of enable/disable events — out of scope

## Decisions

### Decision 1: `is_enabled` column on `group_servers` table
**Choice**: Add `BOOLEAN NOT NULL DEFAULT true` column to `group_servers`.

**Rationale**: This is a per-group-per-server property, not a global server property. The `group_servers` join table is the correct place. Default `true` ensures backward compatibility — all existing assignments remain enabled.

**Alternative considered**: Soft-delete with `disabled_at` timestamp. Rejected — boolean is simpler, and we don't need to track when it was disabled.

### Decision 2: Filter at query level, not application level
**Choice**: Add `AND gs.is_enabled = true` to the proxy's SQL query that builds `GroupConfig.servers`. Disabled servers never enter the Redis cache.

**Rationale**: Filtering at the DB level is the cleanest approach. The proxy code doesn't need to know about disabled servers at all — they simply don't exist in the config. This also means the existing `config.servers.is_empty()` check naturally handles the "all servers disabled" case (returns 429).

**Alternative considered**: Include disabled servers in cache and filter in Rust code. Rejected — adds unnecessary complexity to the proxy hot path with no benefit.

### Decision 3: Admin detail query returns all servers (including disabled)
**Choice**: The admin group detail endpoint returns all servers with `is_enabled` field. Only the proxy query filters.

**Rationale**: Admin UI needs to see and toggle disabled servers. The admin query at `routes/admin/groups.rs:159` returns all servers; the proxy query at `routes/proxy.rs:50` filters to enabled only.

### Decision 4: Toggle via existing `UpdateAssignment` endpoint
**Choice**: Extend `PUT /api/admin/groups/{group_id}/servers/{server_id}` to accept `is_enabled` in the `UpdateAssignment` body, using the existing COALESCE pattern.

**Rationale**: No new endpoint needed. The existing update endpoint already handles partial updates for `priority` and `model_mappings`. Adding `is_enabled` follows the same pattern. Cache invalidation is already handled.

## Risks / Trade-offs

- **[All servers disabled]** → Proxy returns 429 "All upstream servers unavailable". This is the existing behavior for empty server lists — no new handling needed. Admin UI could show a warning, but this is out of scope for this change.
- **[Migration on production]** → `ALTER TABLE ADD COLUMN ... DEFAULT true` is safe in PostgreSQL — it's a metadata-only change for non-null columns with defaults (Pg 11+). No table rewrite needed.
