## Context

Viber Router is a relay proxy with a Vue 3 + Quasar admin SPA and a Rust Axum backend. All admin users share the same `ADMIN_TOKEN`. When one collaborator creates an upstream server with sensitive credentials (base URL, API key), other collaborators with admin access can see those credentials.

Current data model: `servers(id, name, short_id, base_url, api_key, created_at, updated_at)`. The proxy engine queries this table to forward requests — it must always have real credentials available.

## Goals / Non-Goals

**Goals:**
- Allow any admin to protect a server's credentials with a password
- Hide `base_url` and `api_key` from other admins when protected
- Require password to reveal credentials or trigger edit/download actions
- Session-level unlock state (in-memory, resets on page refresh)
- Minimal backend surface area — single verify endpoint

**Non-Goals:**
- Per-user credential isolation in the proxy engine (proxy always has real credentials)
- Long-lived credential access tokens or session persistence
- Changing the auth model (all admins still share `ADMIN_TOKEN`)
- Protecting group-level API keys or sub-keys
- Rate-limiting or lockout on wrong password attempts

## Decisions

### 1. Password hashing: SHA-256 (no external crate)

**Decision**: Store `sha256(password)` hex string in the `password_hash` column. The verify endpoint compares `sha256(input) == stored_hash`.

**Rationale**: User explicitly requested "đơn giản thôi, không cần quá cầu kỳ". SHA-256 is available in Rust's std library (`sha2` crate, already a transitive dependency via other crates). No new cryptographic crate dependency needed. A full argon2/argon2id setup would be overkill for admin panel credential hiding.

**Alternatives considered**:
- bcrypt: Added dependency, slower, same threat model for admin-only UI
- argon2: Overkill for this use case
- Plaintext: Completely unacceptable — hash stored in DB is readable from DB backups

**Open question**: Should the verify endpoint be rate-limited per server_id? Given this is an internal admin tool, deferred for now.

### 2. Verify endpoint response: returns full credentials

**Decision**: `POST /api/admin/servers/{id}/verify-password` with body `{ "password": "..." }` returns `{ "base_url": "...", "api_key": "..." }` on success.

**Rationale**: The frontend needs both `base_url` and `api_key` to populate the edit form and generate cURL commands. A single round-trip to verify is simpler than two endpoints or a session token.

**Alternatives considered**:
- Session token approach: server returns a temporary token, frontend stores it. Rejected — adds session management complexity for marginal benefit (session lasts until page refresh anyway).
- Two separate endpoints: verify only returns boolean. Rejected — wastes a round-trip.

### 3. Unlock state: Pinia store `unlockedServers: Set<string>`

**Decision**: Store unlocked server IDs in the Pinia `servers` store. Reactivity drives re-renders.

```
// stores/servers.ts
const unlockedServers = ref(new Set<string>());

function isUnlocked(id: string): boolean { return unlockedServers.value.has(id); }

async function unlockServer(id: string, password: string): Promise<boolean> {
  const { data } = await api.post('/api/admin/servers/${id}/verify-password', { password });
  unlockedServers.value.add(id);
  return data; // { base_url, api_key }
}

function lockServer(id: string): void { unlockedServers.value.delete(id); }
```

**Rationale**: Consistent with existing Pinia store patterns in the project. No new context/state mechanism needed.

### 4. Masked display: server name replaces URL

**Decision**: Instead of `"••••••••"`, protected values display as the server's name (e.g. `"OpenAI"`). A lock icon (`🔒`) prefixes the masked value.

**Rationale**: Server name is already visible and non-sensitive. Replacing with name provides context while hiding the sensitive URL. Consistent with the existing `maskKey` function's goal of partial reveal.

### 5. Backend API changes: minimal surface

**Decision**: Extend existing `Server` model + CRUD handlers, add one new handler.

```
src/routes/admin/servers.rs:
  - Server model: add password_hash: Option<String>
  - CreateServer DTO: add password: Option<String>  (plaintext, hashed on insert)
  - UpdateServer DTO: add password: Option<Option<String>>  (None=unchanged, Some(None)=clear, Some(Some)=set)
  - POST /{id}/verify-password: new handler
  - GET /servers list: unchanged (still returns base_url/api_key — frontend masks)
  - GET /servers/{id}: unchanged (frontend masks)
  - PUT /servers/{id}: update password_hash if password field provided
```

**Rationale**: The API itself is not changing its behavior — it still returns all fields. The masking happens at the frontend layer. This keeps the backend change minimal and avoids duplicating sensitive data logic in two places.

### 6. Log masking: server_id → server_name mapping

**Decision**: `LogsPage.vue` fetches the list of protected server IDs from the servers store. When rendering a `FailoverAttempt`, if `attempt.server_id` is in the protected set, render `server_name` instead of `upstream_url`.

```
// In LogsPage.vue render
const isProtected = serversStore.protectedServerIds.has(attempt.server_id);
// Or if unlocked:
const isProtected = serversStore.isUnlocked(attempt.server_id) === false;
```

**Rationale**: The logs API returns the real `upstream_url`. Frontend masks based on server protection state. This avoids changing the logs API schema and keeps log data complete for authorized users.

## Risks / Trade-offs

[Risk] Wrong password entry → no feedback differentiation
→ **Mitigation**: API returns 401 with `{"error": "Incorrect password"}`. Frontend shows inline error in dialog. No lockout for now.

[Risk] Password set on a server, admin doesn't know it
→ **Mitigation**: Any admin can clear/reset the password via the edit dialog. The password protects visibility, not access.

[Risk] Proxy still uses real credentials — protection is UI-only
→ **Mitigation**: This is by design. The proxy must always function with real credentials. Protection applies only to the admin panel.

[Risk] Unlock state in Pinia lost on page refresh
→ **Mitigation**: Deliberately session-only as per user decision ("Nhập mỗi khi cần xem"). No Redis or localStorage persistence.

## Migration Plan

1. Add `password_hash` nullable column to `servers` table (new migration `028`). All existing servers have no password — unprotected by default.
2. Deploy backend with new endpoint.
3. Deploy frontend with masking logic — existing servers show real values (no password set).
4. Any admin can set a password on a server via the edit dialog.
5. Rollback: revert migration (additive column, no data loss on removal).
