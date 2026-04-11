## 1. Database Migration

- [x] 1.1 Create `viber-router-api/migrations/038_create_group_user_agents.sql` with `group_user_agents` table (group_id FK, user_agent TEXT, first_seen_at TIMESTAMPTZ DEFAULT now(), PK (group_id, user_agent), ON DELETE CASCADE)
- [x] 1.2 Add `group_blocked_user_agents` table to migration 038 (group_id FK, user_agent TEXT, created_at TIMESTAMPTZ DEFAULT now(), PK (group_id, user_agent), ON DELETE CASCADE)
- [x] 1.3 Add index on `group_blocked_user_agents(group_id)` to migration 038 ← (verify: migration runs cleanly with `sqlx migrate run`, both tables and index exist, FK constraints correct)

## 2. Cache Layer

- [x] 2.1 Add `add_group_ua(redis: &Pool, group_id: Uuid, user_agent: &str) -> Result<bool>` to `viber-router-api/src/cache.rs` — runs `SADD group:{group_id}:user_agents {ua}`, returns true if return value is 1 ← (verify: function compiles, returns true for new member and false for existing member)

## 3. GroupConfig Model

- [x] 3.1 Add `blocked_user_agents: Vec<String>` field with `#[serde(default)]` to `GroupConfig` in `viber-router-api/src/models/group_server.rs`

## 4. Backend: resolve_group_config

- [x] 4.1 Add query in `resolve_group_config` (in `viber-router-api/src/routes/proxy.rs`) to fetch `user_agent` rows from `group_blocked_user_agents` for the group, after the `key_allowed_models` query
- [x] 4.2 Populate `GroupConfig.blocked_user_agents` from the query result ← (verify: GroupConfig serialized to Redis includes blocked_user_agents; old cached entries without the field deserialize with empty Vec)

## 5. Backend: Proxy UA Recording

- [x] 5.1 In `proxy_handler` (proxy.rs), after `resolve_group_config`, extract `User-Agent` header and normalize absent/empty to `"(empty)"`
- [x] 5.2 Spawn a `tokio::spawn` task that calls `add_group_ua`; if it returns `Ok(true)`, insert into `group_user_agents` with `ON CONFLICT DO NOTHING`; log errors but do not propagate ← (verify: new UA appears in group_user_agents after a proxy request; second request with same UA does not insert again; proxy latency unaffected)

## 6. Backend: Proxy UA Block Check

- [x] 6.1 In `proxy_handler`, after the `is_active` check and before the `servers.is_empty()` check, extract UA (same normalization as recording), check against `config.blocked_user_agents` with exact match
- [x] 6.2 If matched, return HTTP 403 with path-appropriate error format, `permission_error` type, message `"Access denied"` ← (verify: blocked UA returns 403 with correct body; non-blocked UA proceeds normally; "(empty)" blocks requests with no UA header)

## 7. Backend: Admin API — group_user_agents.rs

- [x] 7.1 Create `viber-router-api/src/routes/admin/group_user_agents.rs` with handler `list_group_user_agents` — GET `/`, queries `group_user_agents` ordered by `first_seen_at DESC`, returns JSON array
- [x] 7.2 Add handler `list_group_blocked_user_agents` — GET `/blocked`, queries `group_blocked_user_agents` ordered by `created_at DESC`, returns JSON array
- [x] 7.3 Add handler `add_group_blocked_user_agent` — POST `/blocked`, body `{user_agent: String}`, inserts into `group_blocked_user_agents`, calls `invalidate_group_all_keys`, returns 201
- [x] 7.4 Add handler `remove_group_blocked_user_agent` — DELETE `/blocked`, body `{user_agent: String}`, deletes from `group_blocked_user_agents`, calls `invalidate_group_all_keys`, returns 204
- [x] 7.5 Build the router function returning the nested Router for these four handlers ← (verify: all four endpoints respond correctly; POST returns 201; DELETE returns 204; cache invalidated after mutations)

## 8. Backend: Register Module and Nest Router

- [x] 8.1 Add `pub mod group_user_agents;` to `viber-router-api/src/routes/admin/mod.rs`
- [x] 8.2 Nest the `group_user_agents` router under `/{group_id}/user-agents` in `viber-router-api/src/routes/admin/groups.rs` ← (verify: `cargo check` passes; routes accessible at correct paths)

## 9. Frontend: Store Functions

- [x] 9.1 Add `fetchGroupUserAgents(groupId)` to `src/stores/groups.ts` — GET `/api/admin/groups/{id}/user-agents`
- [x] 9.2 Add `fetchGroupBlockedUserAgents(groupId)` — GET `/api/admin/groups/{id}/user-agents/blocked`
- [x] 9.3 Add `addGroupBlockedUserAgent(groupId, userAgent)` — POST with body `{user_agent: userAgent}`
- [x] 9.4 Add `removeGroupBlockedUserAgent(groupId, userAgent)` — DELETE with body `{user_agent: userAgent}` ← (verify: all four functions present and typed correctly; `just check` passes)

## 10. Frontend: User Agents Tab UI

- [x] 10.1 Add "User Agents" tab after "Spam" tab in `src/pages/GroupDetailPage.vue` (tab button and tab panel)
- [x] 10.2 Add reactive state: `userAgents` (recorded list), `blockedUserAgents`, `uaLoading`, `selectedUa` (for q-select input)
- [x] 10.3 Add `q-select` with `use-input`, `use-chips`, filtering from `userAgents` list, allowing custom input; add "Add" button wired to `addGroupBlockedUserAgent`
- [x] 10.4 Render blocked UAs as `q-chip` elements with delete (x) button wired to `removeGroupBlockedUserAgent`
- [x] 10.5 Show loading state while `uaLoading` is true; show empty state message when `blockedUserAgents` is empty
- [x] 10.6 Watch `activeTab` to load data when "User Agents" tab is activated; refresh both lists after add/remove
- [x] 10.7 Add success/error q-notify calls for add and remove operations ← (verify: tab renders; autocomplete filters recorded UAs; add/remove work end-to-end; notifications appear; `just check` passes)
