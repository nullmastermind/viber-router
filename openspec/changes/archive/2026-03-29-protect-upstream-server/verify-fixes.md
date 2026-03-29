## [2026-03-29] Round 1 (from spx-apply auto-verify)

### spx-uiux-verifier
- Fixed: `unlockDialog` in `ServersPage.vue` was `void`, not `async Promise<void>`. If `unlockServer` threw, the internal Promise never resolved. Since `openDialog` called it without `await`, the form would silently open even on wrong password. Both `onOk` (success + failure) and `onCancel` now call `resolve()`.
- Fixed: `unlockDialog` in `GroupDetailPage.vue` — same root cause as above. Fixed to `async Promise<void>` with all paths calling `resolve()`.
- Fixed: `LogsPage.vue` collapsed summary row's `server_name` column showed bare `attempt.server_name` — locked servers were not masked there, only in the expanded failover timeline. Added `isProtected`/`isUnlocked` guards to both the failover-chain span and the single-server `v-else` branch.

### spx-arch-verifier
- Fixed: `serde::Serialize` was not imported in `servers.rs`, causing the `VerifyPasswordResponse` derive to fail.
- Fixed: `verify_password` used `server.password_hash.unwrap()` which panics if `None`. Changed to `if let Some(expected_hash) = &server.password_hash` pattern to avoid panicking.

### spx-test-verifier
- No issues (no test files modified).

### spx-verifier
- Fixed: `is_protected?: boolean` field removed from `FailoverAttempt` interface in `LogsPage.vue` — the field was defined but never read; all masking logic uses `serversStore.isProtected(attempt.server_id)` directly instead.

## [2026-03-29] Round 2 (security fix — critical: API returned credentials in network responses)

### spx-verifier / security
- **CRITICAL FIX**: Frontend-only masking (`v-if` in templates) was completely bypassable — anyone could open DevTools Network tab and see real `base_url`/`api_key` in JSON responses. This was a fundamental design flaw.
- **Fix**: Moved masking to the backend. Added `ServerResponse` type that returns `base_url: null` and `api_key: null` for any server that has `password_hash` set AND is not in the current session's unlocked set.
- Added `UnlockedServers = Arc<RwLock<HashSet<Uuid>>>` to `AppState` (in-memory per-process session). Updated `main.rs` to initialize it and pass it through state.
- `verify_password` handler now inserts the server ID into `unlocked_servers` on success. All subsequent GET/list responses for that server ID return real credentials until the server process restarts.
- `delete_server` removes the server ID from the unlocked set.
- `create_server`, `list_servers`, `get_server`, `update_server` now all use `ServerResponse::from_server()` which checks the unlocked set.
- Groups endpoint (`GET /groups/{id}`) also revealed server credentials. Updated `AdminGroupServerDetail` to include `password_hash` field. Groups handler now reads `unlocked_servers` and masks `base_url`/`api_key` for locked servers.
- Removed frontend-only masking logic (no longer needed — API enforces it). The `🔒 + name` display in templates remains as UX indicator.

### spx-arch-verifier
- Fixed: `std::collections::HashSet` was imported but unused in `groups.rs` after restructuring — removed.
- Fixed: `uuid::Uuid` was imported but unused in `main.rs` — removed.

### Frontend updates
- `Server.base_url` in `stores/servers.ts` changed from `string` to `string | null` (API can now return null).
- `GroupServerDetail.base_url` in `stores/groups.ts` changed from `string` to `string | null` (API can now return null).
- `unlockDialog` in `ServersPage.vue` and `GroupDetailPage.vue` now re-fetches after unlock so the API returns real credentials and the UI updates.
- `openDialog`/`openEditServer` in both pages now look up the fresh server data from the store after unlocking, to populate the form with real credentials.
- Removed unused `is_protected?: boolean` from `FailoverAttempt` interface in `LogsPage.vue`.

