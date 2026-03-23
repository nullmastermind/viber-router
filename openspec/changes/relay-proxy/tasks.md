## 1. Database Migrations

- [x] 1.1 Add `sqlx-cli` dev dependency and `sqlx::migrate` feature to Cargo.toml; add `reqwest` with `json` and `stream` features; add `uuid` with `v4` and `serde` features; add `rand` for key generation
- [x] 1.2 Create migration `001_create_servers` — table with id (uuid PK), name (text NOT NULL), base_url (text NOT NULL), api_key (text NOT NULL), created_at, updated_at
- [x] 1.3 Create migration `002_create_groups` — table with id (uuid PK), name (text NOT NULL), api_key (text NOT NULL UNIQUE), failover_status_codes (jsonb NOT NULL DEFAULT '[429,500,502,503]'), is_active (boolean NOT NULL DEFAULT true), created_at, updated_at; indexes on api_key, name, is_active
- [x] 1.4 Create migration `003_create_group_servers` — table with group_id (uuid FK), server_id (uuid FK), priority (integer NOT NULL), model_mappings (jsonb NOT NULL DEFAULT '{}'), created_at; PK on (group_id, server_id); ON DELETE CASCADE for group_id
- [x] 1.5 Embed migrations in main.rs using `sqlx::migrate!()` and run before binding listener ← (verify: all 3 tables created correctly, indexes exist, FK constraints work, app starts clean on empty DB)

## 2. Config & AppState Updates

- [x] 2.1 Add `admin_token` field to Config struct, required from ADMIN_TOKEN env var (fail on missing)
- [x] 2.2 Update AppState to include `admin_token: String` alongside existing db and redis fields

## 3. Server Management (Backend)

- [x] 3.1 Create `src/models/` module with Server, Group, GroupServer, GroupConfig structs (serde Serialize/Deserialize)
- [x] 3.2 Create `src/routes/admin/servers.rs` — CRUD handlers: create_server (POST), list_servers (GET with pagination + search), get_server (GET by id), update_server (PUT), delete_server (DELETE with 409 if assigned to groups)
- [x] 3.3 Wire server admin routes under `/api/admin/servers` ← (verify: all CRUD operations work, delete rejects when server is assigned to groups)

## 4. Group Management (Backend)

- [x] 4.1 Create `src/routes/admin/groups.rs` — CRUD handlers: create_group (POST, auto-gen sk-vibervn-* key), list_groups (GET with server-side pagination, search, filter by is_active, filter by server_id, sort by created_at), get_group (GET by id with servers array), update_group (PUT), delete_group (DELETE)
- [x] 4.2 Add regenerate_key handler (POST `/groups/{id}/regenerate-key`)
- [x] 4.3 Add bulk handlers: bulk_activate, bulk_deactivate, bulk_delete, bulk_assign_server
- [x] 4.4 Wire group admin routes under `/api/admin/groups` ← (verify: pagination returns correct total/pages, search is case-insensitive, filter by server_id works, bulk ops invalidate cache)

## 5. Group-Server Assignment (Backend)

- [x] 5.1 Create `src/routes/admin/group_servers.rs` — handlers: assign_server (POST), update_assignment (PUT), remove_server (DELETE), reorder_priorities (PUT)
- [x] 5.2 Wire group-server routes under `/api/admin/groups/{group_id}/servers` ← (verify: reorder sets priorities correctly, duplicate assignment returns 409)

## 6. Admin Auth (Backend)

- [x] 6.1 Create `src/middleware/admin_auth.rs` — Axum middleware that checks `Authorization: Bearer <token>` against AppState.admin_token; returns 401 on missing/invalid
- [x] 6.2 Create login handler at POST `/api/admin/login` (accepts token in body, no auth middleware)
- [x] 6.3 Apply admin_auth middleware to all `/api/admin/*` routes except `/api/admin/login` ← (verify: unauthenticated requests get 401, login works without auth header, valid token passes through)

## 7. Config Cache (Backend)

- [x] 7.1 Create `src/cache.rs` — functions: get_group_config(redis, api_key) → Option<GroupConfig>, set_group_config(redis, api_key, config), invalidate_group_config(redis, api_key), invalidate_groups_by_server(redis, db, server_id)
- [x] 7.2 Integrate cache invalidation into all admin write handlers (server update, group CRUD, group-server changes) ← (verify: cache is populated on first proxy request, invalidated on every admin write, Redis failure falls back to DB)

## 8. Proxy Engine (Backend)

- [x] 8.1 Create `src/routes/proxy.rs` — catch-all handler for `/v1/*` that: extracts x-api-key, looks up group config (cache → DB), validates group is active
- [x] 8.2 Implement request body buffering and model name transformation (parse JSON, replace model field per group-server mapping, re-serialize)
- [x] 8.3 Implement upstream forwarding with reqwest — swap api_key header, preserve method/path/query/headers/body, forward to upstream server
- [x] 8.4 Implement failover waterfall — iterate servers by priority, on failover status code or connection error try next, on all-fail return 429 + Retry-After:30 + Anthropic error JSON
- [x] 8.5 Implement SSE streaming passthrough — detect streaming response, pipe upstream SSE stream to client response
- [x] 8.6 Mount proxy routes and verify end-to-end flow ← (verify: non-streaming proxy works, streaming SSE passthrough works, failover triggers on configured status codes, model name transformation applies correctly, all-fail returns 429 with correct body and headers)

## 9. Frontend — Auth & Layout

- [x] 9.1 Update `src/boot/axios.ts` — set baseURL to backend API, add request interceptor to attach Authorization header from localStorage
- [x] 9.2 Create `src/pages/LoginPage.vue` — token input, login button, error display, stores token in localStorage on success
- [x] 9.3 Create auth navigation guard in router — redirect to login if no token, redirect to servers page after login
- [x] 9.4 Update `MainLayout.vue` — replace boilerplate with admin sidebar (Servers, Groups links) and logout button ← (verify: login flow works, unauthenticated access redirects to login, logout clears token)

## 10. Frontend — Servers Page

- [x] 10.1 Create `src/stores/servers.ts` — Pinia store with CRUD actions calling admin API
- [x] 10.2 Create `src/pages/ServersPage.vue` — QTable listing servers (name, base_url, api_key), search field, Add/Edit/Delete actions with QDialog forms ← (verify: CRUD operations reflect in table, delete shows error for assigned servers)

## 11. Frontend — Groups Page

- [x] 11.1 Create `src/stores/groups.ts` — Pinia store with paginated list, search, filter, bulk actions, CRUD
- [x] 11.2 Create `src/pages/GroupsPage.vue` — QTable with server-side pagination, search input, status filter dropdown, server filter dropdown, checkbox selection, bulk action buttons (Activate, Deactivate, Delete, Assign Server)
- [x] 11.3 Create `src/pages/GroupDetailPage.vue` — group properties editor (name, failover codes, status toggle), API key display with copy/regenerate, server list with drag-to-reorder (QList with sortable), model mapping editor per server, add/remove server ← (verify: pagination shows correct page/total, search filters server-side, bulk ops update all selected, drag reorder persists priorities, model mapping CRUD works)

## 12. Frontend — Routing

- [x] 12.1 Update `src/router/routes.ts` — add routes: /login, /servers, /groups, /groups/:id; set / to redirect to /servers ← (verify: all routes resolve, navigation guard protects admin pages)

## 13. Testing

- [x] 13.1 Backend unit tests: model name transformation logic, API key generation, failover status code matching
- [x] 13.2 Backend integration tests: proxy failover waterfall (mock upstream servers), admin CRUD endpoints, cache invalidation flow ← (verify: all tests pass, proxy failover correctly waterfalls through servers, cache is invalidated on writes)
