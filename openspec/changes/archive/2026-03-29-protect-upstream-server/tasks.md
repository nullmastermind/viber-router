## 1. Database

- [x] 1.1 Create migration `028_add_server_password.sql` adding `password_hash TEXT` nullable column to `servers` table ← (verify: migration runs without errors, column is nullable, no existing data affected)

## 2. Backend — Models & Types

- [x] 2.1 Add `password_hash: Option<String>` field to `Server` struct in `src/models/server.rs` ← (verify: struct includes new field, serde derive correct)
- [x] 2.2 Add `password: Option<String>` to `CreateServer` DTO ← (verify: create handler hashes and stores password)
- [x] 2.3 Add `password: Option<Option<String>>` to `UpdateServer` DTO (None=unchanged, Some(None)=clear, Some(Some)=set) ← (verify: update handler handles all three cases)
- [x] 2.4 Update `create_server` handler to hash incoming password with SHA-256 and store ← (verify: server with password returns password_hash, server without password has null)
- [x] 2.5 Update `update_server` handler to handle password update/clear ← (verify: setting password stores hash, clearing sets null)

## 3. Backend — Verify Endpoint

- [x] 3.1 Add `VerifyPassword` request body struct ← (verify: deserializes from `{"password": "..."}`)
- [x] 3.2 Add `VerifyPasswordResponse` struct `{base_url: String, api_key: Option<String>}` ← (verify: serializes correctly)
- [x] 3.3 Implement `verify_password` handler: lookup server, check password_hash, compare SHA-256, return credentials on match ← (verify: correct password returns 200, wrong returns 401, no password returns 200, not found returns 404)
- [x] 3.4 Register new route `router().route("/{id}/verify-password", post(verify_password))` ← (verify: endpoint accessible at POST /api/admin/servers/{id}/verify-password)

## 4. Frontend — Servers Store

- [x] 4.1 Add `protectedServerIds: Set<string>` to `stores/servers.ts` — set of server IDs that have a password hash ← (verify: populated from server list response, drives mask logic)
- [x] 4.2 Add `unlockedServers: Set<string>` to `stores/servers.ts` — set of currently unlocked server IDs ← (verify: cleared on store initialization, resets unlock state)
- [x] 4.3 Add `isProtected(id: string): boolean` method ← (verify: returns true iff server has password_hash)
- [x] 4.4 Add `isUnlocked(id: string): boolean` method ← (verify: returns true iff in unlockedServers)
- [x] 4.5 Add `async unlockServer(id: string, password: string): Promise<{base_url: string, api_key: string|null}>` — POST to verify endpoint, add to unlockedServers on success, throw on 401 ← (verify: 401 rejected with error, success adds to set, server stays locked on failure)
- [x] 4.6 Add `lockServer(id: string): void` — removes from unlockedServers ← (verify: removes from set, UI re-masks values immediately)
- [x] 4.7 Update `fetchServers` to populate `protectedServerIds` from response ← (verify: any server with password_hash populates the protected set)

## 5. Frontend — ServersPage

- [x] 5.1 Update `Server` interface to include `password_hash: string | null` ← (verify: type matches API response)
- [x] 5.2 In `base_url` column template: if `isProtected(row.id)` → show `🔒 ${row.name}`, else show real URL ← (verify: locked shows name+icon, unlocked shows real URL)
- [x] 5.3 In `api_key` column template: if `isProtected(row.id)` → show `🔒 ${row.name}`, else show real/masked key ← (verify: consistent with base_url masking)
- [x] 5.4 Add `unlockDialog(serverId: string, onSuccess: () => void)` helper function using `$q.dialog` — shows password input, calls `store.unlockServer`, shows error on failure ← (verify: error shown on wrong password, dialog stays open, success closes dialog)
- [x] 5.5 In `openDialog(server?)`: if `server && isProtected(server.id) && !isUnlocked(server.id)` → call unlockDialog first; if cancelled, abort open; if success → proceed with real values ← (verify: protected locked server → prompt → cancel does not open dialog; success opens with real values)
- [x] 5.6 Add "Protect password" `q-input type="password"` to create and edit dialogs ← (verify: create with password → server is protected; create without → server is unprotected)
- [x] 5.7 In `saveServer`: send `password` field (empty string = no password change on edit, omit on create if empty) ← (verify: backend receives correct password payload)

## 6. Frontend — GroupDetailPage

- [x] 6.1 In Servers tab server list: if `isProtected(s.server_id) && !isUnlocked(s.server_id)` → show `🔒 ${s.server_name}` instead of `s.base_url` caption ← (verify: protected locked server shows lock+name, unlocked shows real URL)
- [x] 6.2 Add lock icon button next to each protected server row (same level as edit button) that triggers `unlockDialog` ← (verify: icon only shows on protected servers, clicking opens prompt)
- [x] 6.3 In `openEditServer(s)`: if `isProtected(s.server_id) && !isUnlocked(s.server_id)` → call `unlockDialog` first; if cancelled → abort; if success → proceed with real values pre-filled ← (verify: same flow as ServersPage)
- [x] 6.4 Update `keyBuilderEntries` — `defaultKey` display: if `isProtected(s.server_id) && !isUnlocked(s.server_id)` → show `🔒 Protected` instead of real api_key ← (verify: key builder shows lock for protected servers)

## 7. Frontend — LogsPage

- [x] 7.1 Update `FailoverAttempt` interface: add `is_protected?: boolean` (populated by frontend based on store) ← (verify: type matches usage in template) — **Note:** field removed; masking uses `serversStore.isProtected(attempt.server_id)` directly
- [x] 7.2 In failover chain display: if `isProtected(attempt.server_id) && !isUnlocked(attempt.server_id)` → show `🔒 ${attempt.server_name}` instead of `upstream_url` ← (verify: protected server shows lock+name in chain)
- [x] 7.3 Add `unlockDialog` to LogsPage scope (same pattern as ServersPage, can extract to shared composable later) ← (verify: prompt appears, wrong password shows error, success enables cURL download)
- [x] 7.4 In `downloadCurl(attempt, log)`: if `isProtected(attempt.server_id) && !isUnlocked(attempt.server_id)` → call `unlockDialog` first; if cancelled → abort; if success → generate cURL with real `upstream_url` ← (verify: protected locked → prompt → success generates real URL; wrong password shows error, no download)

## 8. Verify

- [x] 8.1 Run `just check` — fix any type errors or lint errors ← (verify: `bun run lint` passes, `cargo clippy -- -D warnings` passes, no type errors)
- [ ] 8.2 Manual smoke test: create server with password → list shows lock+name → unlock → real values shown → refresh → back to locked ← (verify: full flow works end-to-end)
