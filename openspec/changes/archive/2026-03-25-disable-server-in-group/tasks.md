## 1. Database Migration

- [x] 1.1 Create migration file adding `is_enabled BOOLEAN NOT NULL DEFAULT true` column to `group_servers` table

## 2. Backend Models

- [x] 2.1 Add `is_enabled: bool` field to `GroupServer` struct in `models/group_server.rs`
- [x] 2.2 Add `is_enabled: bool` field to `GroupServerDetail` struct in `models/group_server.rs`
- [x] 2.3 Add `is_enabled: Option<bool>` field to `UpdateAssignment` struct in `models/group_server.rs`

## 3. Backend API

- [x] 3.1 Update `assign_server` handler in `routes/admin/group_servers.rs` to include `is_enabled` in INSERT query
- [x] 3.2 Update `update_assignment` handler in `routes/admin/group_servers.rs` to handle `is_enabled` via COALESCE pattern
- [x] 3.3 Update group detail query in `routes/admin/groups.rs` to SELECT `gs.is_enabled` in GroupServerDetail ← (verify: admin API returns is_enabled for all servers, update correctly toggles is_enabled and invalidates cache)

## 4. Proxy Query

- [x] 4.1 Add `AND gs.is_enabled = true` filter to the server query in `routes/proxy.rs` ← (verify: disabled servers are excluded from failover chain, all-disabled returns 429)

## 5. Frontend

- [x] 5.1 Add `is_enabled: boolean` to `GroupServerDetail` interface in `stores/groups.ts`
- [x] 5.2 Add inline `q-toggle` switch to each server row in `GroupDetailPage.vue` that calls `updateAssignment` with `{ is_enabled }` on change
- [x] 5.3 Apply dimmed opacity and strikethrough styling to disabled server rows in `GroupDetailPage.vue` ← (verify: toggle works without confirm, visual states match spec, `just check` passes)
