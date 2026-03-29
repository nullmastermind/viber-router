## Why

Multiple collaborators share the same Viber Router admin panel. When one collaborator adds an upstream server with a private base URL and API key, other collaborators can see those credentials. There is no way to protect sensitive upstream server credentials from other admin users.

This change adds per-server password protection so any collaborator can protect their upstream servers from being viewed or used by others in the shared admin panel.

## What Changes

- **New `password_hash` column** on the `servers` table (optional, SHA-256). Any admin can set/change a server's password.
- **New backend endpoint**: `POST /api/admin/servers/{id}/verify-password` — accepts a password, returns the server's real `base_url` and `api_key` on success. Returns 401 on wrong password.
- **Frontend masking**: Anywhere `base_url` or `api_key` is displayed for a password-protected server, the real value is replaced with the server name (e.g. `"OpenAI"`). A lock icon indicates the server is protected.
- **Frontend unlock flow**: Clicking the lock icon opens a password prompt. On correct password, real values are revealed in that session (persists until page refresh). The unlock state is stored in the Pinia `servers` store as `unlockedServers: Set<string>`.
- **Edit and download flows protected**: Editing a protected server's details or downloading a protected server's cURL from a log row requires entering the password first. After successful verification, the edit form pre-fills real values.
- **No proxy engine changes**: The proxy router continues to use real `base_url`/`api_key` for all upstream forwarding. Protection only applies to the admin UI layer.
- **No auth changes**: All admins share the same `ADMIN_TOKEN`. Password protection separates credential visibility, not access control.

## Capabilities

### New Capabilities

- `server-protection`: Per-server password protection in the admin UI. When a server has a password set, its `base_url` and `api_key` are hidden from other admin users. Verification unlocks real values for the current session.

### Modified Capabilities

(No existing requirement changes. The server-management spec defines CRUD behavior which is unchanged — this adds a new protection layer on top.)

## Impact

- **Database**: New nullable `password_hash TEXT` column in `servers` table; new migration file.
- **Backend**: New `verify_password` handler in `routes/admin/servers.rs`; `Server` model and `CreateServer`/`UpdateServer` DTOs updated to include `password_hash`.
- **Frontend**:
  - `stores/servers.ts` — add `unlockedServers: Set<string>` reactive state and `isUnlocked(id)` / `unlockServer(id, password)` / `lockServer(id)` methods.
  - `pages/ServersPage.vue` — mask `base_url`/`api_key` with server name; add "Protect password" field in create/edit dialogs; lock icon unlocks inline values.
  - `pages/GroupDetailPage.vue` — in the Servers tab, mask `base_url` in server list items; protect edit dialog with password prompt.
  - `pages/LogsPage.vue` — in expanded log rows, mask `upstream_url` in the failover chain with `server_name`; protect "Download cURL" button with password prompt.
- **Proxy engine**: No changes.
- **Dependencies**: No new crates or npm packages.
