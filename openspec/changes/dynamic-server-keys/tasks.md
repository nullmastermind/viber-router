## 1. Database Migration

- [x] 1.1 Create migration `004_dynamic_server_keys` — ALTER `servers` table: add `short_id SERIAL UNIQUE NOT NULL`, ALTER `api_key` to nullable (`ALTER COLUMN api_key DROP NOT NULL`)
- [x] 1.2 Update `Server` struct in `models/server.rs` — `api_key: Option<String>`, add `short_id: i32` ← (verify: struct matches migration, all queries compile)

## 2. Dynamic Key Parser

- [x] 2.1 Create `src/routes/key_parser.rs` module — implement `parse_api_key(header: &str) -> ParsedKey { group_key: String, dynamic_keys: HashMap<i32, String> }`. Split by `-rsv-`, parse short_id and server key from each segment. On any parse failure, return entire string as group_key with empty dynamic_keys map.
- [x] 2.2 Add unit tests for parser — plain key, single dynamic key, multiple dynamic keys, malformed segment (non-numeric short_id), segment with no key after short_id ← (verify: all 5 spec scenarios from dynamic-key-parsing/spec.md pass)

## 3. Backend Model Updates

- [x] 3.1 Update `CreateServer` — make `api_key` optional (`Option<String>`)
- [x] 3.2 Update `GroupServerDetail` — add `short_id: i32`, change `api_key` to `Option<String>`
- [x] 3.3 Update `GroupConfig` — add `short_id: i32` to server entries so proxy can match dynamic keys
- [x] 3.4 Update all SQL queries that SELECT from `servers` or JOIN with `servers` to include `short_id` and handle nullable `api_key` ← (verify: `cargo check` passes, no compile errors from sqlx)

## 4. Key Resolution in Proxy

- [x] 4.1 Update `proxy_handler` in `routes/proxy.rs` — call `parse_api_key` on the `x-api-key` header, use extracted `group_key` for group lookup
- [x] 4.2 Implement key resolution logic in the failover loop — for each server: check dynamic_keys by short_id first, then server default api_key, skip if neither exists
- [x] 4.3 Handle all-servers-skipped case — return HTTP 401 `{"type":"error","error":{"type":"authentication_error","message":"No server keys configured"}}`
- [x] 4.4 Update cache module — `GroupConfig.servers` entries now carry `short_id` and `Option<String>` api_key, update serialization ← (verify: proxy handles plain key, dynamic key, skip, and all-skipped scenarios correctly)

## 5. Admin API Updates

- [x] 5.1 Update `create_server` handler — accept optional `api_key`, bind as nullable
- [x] 5.2 Update `update_server` handler — allow setting `api_key` to null
- [x] 5.3 Update `list_servers` and `get_server` — ensure `short_id` is included in response
- [x] 5.4 Update group-server detail query — JOIN to include `short_id`, handle nullable `api_key` ← (verify: all admin API endpoints return `short_id`, accept optional `api_key`)

## 6. Frontend — Server Short ID Display

- [x] 6.1 Update `Server` interface in `stores/servers.ts` — add `short_id: number`, make `api_key` optional
- [x] 6.2 Update `GroupServerDetail` interface in `stores/groups.ts` — add `short_id: number`, make `api_key` optional
- [x] 6.3 Update `ServersPage.vue` — add Short ID column with copy button, make api_key field optional in create/edit form
- [x] 6.4 Update `GroupDetailPage.vue` — show `short_id` with copy button next to each server in the server list
- [x] 6.5 Update `GroupsPage.vue` — if server short_id is visible in bulk assign or server filter, include it ← (verify: short_id visible and copyable on all pages, server creation works without api_key)

## 7. Lint & Type Check

- [x] 7.1 Run `just check` — fix any clippy warnings, biome lint errors, or TypeScript errors ← (verify: `just check` passes clean)
