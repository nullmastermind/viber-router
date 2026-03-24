## 1. Database Migration

- [x] 1.1 Create migration file `viber-router-api/migrations/008_count_tokens_default_server.sql` with: `ALTER TABLE groups ADD COLUMN count_tokens_server_id UUID REFERENCES servers(id) ON DELETE SET NULL` and `ALTER TABLE groups ADD COLUMN count_tokens_model_mappings JSONB NOT NULL DEFAULT '{}'`
- [x] 1.2 Apply migration to dev database ← (verify: columns exist, FK constraint works, ON DELETE SET NULL triggers correctly)

## 2. Backend Models

- [x] 2.1 Add `count_tokens_server_id: Option<Uuid>` and `count_tokens_model_mappings: serde_json::Value` to `Group` struct in `viber-router-api/src/models/group.rs`
- [x] 2.2 Add same fields to `GroupListItem` struct (required because `list_groups` uses `SELECT g.*`)
- [x] 2.3 Add `count_tokens_server_id: Option<Option<Uuid>>` and `count_tokens_model_mappings: Option<serde_json::Value>` to `UpdateGroup` struct
- [x] 2.4 Create `CountTokensServer` struct in `viber-router-api/src/models/group_server.rs` with fields: `server_id: Uuid`, `short_id: i32`, `server_name: String`, `base_url: String`, `api_key: Option<String>`, `model_mappings: serde_json::Value`
- [x] 2.5 Add `count_tokens_server: Option<CountTokensServer>` to `GroupConfig` struct ← (verify: all model structs compile, serde derives present)

## 3. Cache Layer

- [x] 3.1 Extend `invalidate_groups_by_server` query in `viber-router-api/src/cache.rs` to include `OR g.count_tokens_server_id = $1` using a UNION or adjusted WHERE clause ← (verify: invalidation covers both group_servers and count_tokens_server_id references)

## 4. Admin API — Group Update

- [x] 4.1 Update `update_group` handler in `viber-router-api/src/routes/admin/groups.rs` to accept and persist `count_tokens_server_id` and `count_tokens_model_mappings` using the `CASE WHEN` pattern (same as `ttft_timeout_ms`)
- [x] 4.2 Update `get_group` handler to include `count_tokens_server_id` and `count_tokens_model_mappings` in response (already returned via `SELECT *`, but verify struct fields match) ← (verify: PUT with `count_tokens_server_id: "<uuid>"` persists, PUT with `null` clears, GET returns both fields)

## 5. Proxy — Resolve Config

- [x] 5.1 Update `resolve_group_config` in `viber-router-api/src/routes/proxy.rs` to fetch count-tokens server detail when `group.count_tokens_server_id IS NOT NULL`: query `servers` table for `server_id, short_id, name, base_url, api_key`, combine with `group.count_tokens_model_mappings` to build `CountTokensServer`, set on `GroupConfig` ← (verify: GroupConfig cached with count_tokens_server populated when configured, None when not configured)

## 6. Proxy — Count Tokens Routing

- [x] 6.1 In `proxy_handler`, after config resolution and before the failover loop: check if `original_uri.path() == "/v1/messages/count_tokens"` AND `config.count_tokens_server` is Some. If so, attempt the default server using the same request-building logic (key resolution via short_id/api_key, model transform using count_tokens_server.model_mappings, header forwarding)
- [x] 6.2 Handle default server response: if HTTP 200 return response; if failover status code or connection error, record in failover_chain and proceed to waterfall; if non-failover error, return directly
- [x] 6.3 In the failover loop, add skip condition: `if let Some(ref ct) = config.count_tokens_server { if server.server_id == ct.server_id { continue; } }` — only when the path is count_tokens and default was attempted
- [x] 6.4 Ensure logging (emit_log_entry) includes the default server attempt in the failover_chain ← (verify: default server success returns 200, failover status triggers chain with default skipped, connection error triggers chain with default skipped, non-failover error returns directly, normal paths unaffected)

## 7. Frontend — Store Types

- [x] 7.1 Add `count_tokens_server_id: string | null` and `count_tokens_model_mappings: Record<string, string>` to `Group` interface in `src/stores/groups.ts`
- [x] 7.2 Add same fields to `GroupWithServers` interface (inherited from Group, but verify)
- [x] 7.3 Update `updateGroup` action input type to include `count_tokens_server_id?: string | null` and `count_tokens_model_mappings?: Record<string, string>`

## 8. Frontend — Group Detail UI

- [x] 8.1 In `GroupDetailPage.vue`, add a "Count Tokens Server" card section with a `q-select` dropdown populated from `allServers` (all servers in system), with a "None" option to clear. Bind to `group.count_tokens_server_id`
- [x] 8.2 Add a "Count Tokens Model Mappings" editor (reuse the same from/to pattern as existing model mappings dialog) bound to `group.count_tokens_model_mappings`
- [x] 8.3 Wire `saveGroup` to include `count_tokens_server_id` and `count_tokens_model_mappings` in the update payload
- [x] 8.4 Wire `loadGroup` to populate the new fields from API response ← (verify: dropdown shows all servers, selecting a server persists on save, clearing to None persists null, model mappings save/load correctly)

## 9. Validate

- [x] 9.1 Run `just check` — ensure no type errors, lint warnings, or clippy errors ← (verify: clean output from cargo check + cargo clippy + biome lint)
