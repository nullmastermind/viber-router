## 1. Database Migration

- [x] 1.1 Create migration file `viber-router-api/migrations/<timestamp>_add_supported_models_to_group_servers.sql` with `ALTER TABLE group_servers ADD COLUMN supported_models TEXT[] NOT NULL DEFAULT '{}'`
- [x] 1.2 Run migration against the development database and verify the column exists with the correct default <- (verify: `\d group_servers` shows `supported_models text[] not null default '{}'`)

## 2. Backend Models

- [x] 2.1 Add `#[serde(default)] pub supported_models: Vec<String>` to `GroupServerDetail` in `viber-router-api/src/models/group_server.rs`
- [x] 2.2 Add `#[serde(default)] pub supported_models: Vec<String>` to `AdminGroupServerDetail` in the same file
- [x] 2.3 Add `pub supported_models: Option<Vec<String>>` to `UpdateAssignment` in the same file (None = no change, Some(vec) = set)
- [x] 2.4 Add `pub supported_models: Vec<String>` to `AssignServer` in the same file (defaults to empty vec)
- [x] 2.5 Run `cargo check` in `viber-router-api/` and fix any type errors <- (verify: `cargo check` exits 0, all struct usages compile)

## 3. Admin API Queries

- [x] 3.1 Add `gs.supported_models` to the SELECT column list in the admin group detail query in `viber-router-api/src/routes/admin/group_servers.rs`
- [x] 3.2 Add `supported_models` to the INSERT query for `assign_server` handler (bind `body.supported_models`)
- [x] 3.3 Add conditional `supported_models = $N` clause to the UPDATE query in `update_assignment` handler when `body.supported_models` is `Some`
- [x] 3.4 Run `cargo clippy -- -D warnings` in `viber-router-api/` and fix any warnings <- (verify: clippy exits 0, GET and PUT assignment endpoints return `supported_models` in response JSON)

## 4. Proxy Engine

- [x] 4.1 Add `gs.supported_models` to the SELECT column list in the proxy cache-load query in `viber-router-api/src/routes/proxy.rs`
- [x] 4.2 Insert the model filter skip check in the failover loop after the `max_input_tokens` check: if `supported_models` is non-empty, check that the request model is either in the list or is a key in `model_mappings`; if neither, `continue`
- [x] 4.3 Run `cargo clippy -- -D warnings` and confirm the proxy compiles clean <- (verify: a request for a model not in `supported_models` is skipped silently; a request for a model that is a `model_mappings` key is not skipped; empty `supported_models` causes no skip)

## 5. Frontend Store

- [x] 5.1 Add `supported_models: string[]` to the `GroupServerDetail` interface in `src/stores/groups.ts`
- [x] 5.2 Add `supported_models?: string[]` to the `updateAssignment` input type in `src/stores/groups.ts`
- [x] 5.3 Run `bun run lint` and fix any type errors

## 6. Frontend UI

- [x] 6.1 Add a `supported_models` reactive field (initialized from the server's current value) to the edit-server dialog state in `src/pages/GroupDetailPage.vue`
- [x] 6.2 Add a `QSelect` with `use-chips` and `multiple` bound to `supported_models`, populated with model names from the models store, inside the edit-server dialog form
- [x] 6.3 Include `supported_models` in the `updateAssignment` call payload when saving the dialog
- [x] 6.4 Run `bun run lint` and fix any issues <- (verify: edit dialog shows the multi-select chips field; saving updates the value; existing assignments with empty list show no chips)

## 7. Final Check

- [x] 7.1 Run `just check` (full type-check + lint for both frontend and backend) and fix all reported errors <- (verify: `just check` exits 0 with no errors or warnings)
