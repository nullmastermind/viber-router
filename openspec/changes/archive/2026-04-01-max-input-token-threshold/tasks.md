## 1. Database Migration

- [x] 1.1 Create `viber-router-api/migrations/031_add_max_input_tokens.sql` with `ALTER TABLE group_servers ADD COLUMN max_input_tokens INTEGER NULL`
- [x] 1.2 Run `sqlx migrate run` locally to verify migration applies cleanly ← (verify: column exists in group_servers, all existing rows have max_input_tokens=NULL, `cargo check` passes with updated sqlx queries)

## 2. Backend Models

- [x] 2.1 Add `max_input_tokens: Option<i32>` to `GroupServer` struct in `viber-router-api/src/models/group_server.rs`
- [x] 2.2 Add `max_input_tokens: Option<i32>` to `GroupServerDetail` struct (used by proxy cache)
- [x] 2.3 Add `max_input_tokens: Option<i32>` to `AdminGroupServerDetail` struct (used by admin API responses)
- [x] 2.4 Add `max_input_tokens: Option<Option<i32>>` to `UpdateAssignment` struct (outer Option = not provided, inner = null to clear)
- [x] 2.5 Add `max_input_tokens: Option<i32>` to `AssignServer` struct ← (verify: `cargo check` passes with all struct changes)

## 3. Backend Admin API

- [x] 3.1 In `viber-router-api/src/routes/admin/group_servers.rs`, update the `update_assignment` SQL query to include `max_input_tokens` with the same conditional-update pattern used by `max_requests` and other nullable fields
- [x] 3.2 Add `max_input_tokens` bind variables to the query builder in `update_assignment` (update flag + value pair)
- [x] 3.3 Update the `assign_server` INSERT query to include `max_input_tokens` from the `AssignServer` input (defaulting to NULL if not provided)
- [x] 3.4 Update all SELECT queries that return `GroupServerDetail` or `AdminGroupServerDetail` rows to include the `max_input_tokens` column ← (verify: PUT /api/admin/groups/{id}/servers/{id} with max_input_tokens sets and clears the field; GET /api/admin/groups/{id} returns max_input_tokens in server list; cargo clippy passes)

## 4. Backend Proxy — Token Estimation

- [x] 4.1 In `viber-router-api/src/routes/proxy.rs`, after reading and buffering the request body, add a function `estimate_input_tokens(body: &[u8]) -> Option<usize>` that: parses the body as JSON, strips image content blocks from messages, serializes back to string, returns `Some(len / 4)` or `None` on parse failure
- [x] 4.2 Call `estimate_input_tokens` once before the failover loop and bind the result to a local `estimated_tokens: Option<usize>` variable ← (verify: estimation function handles text-only, mixed text+image, and non-JSON bodies correctly per spec scenarios)

## 5. Backend Proxy — Failover Skip Logic

- [x] 5.1 In the failover waterfall loop (around line 722 of `proxy.rs`), after the rate limit check block, add a `max_input_tokens` check: if `server.max_input_tokens` is `Some(limit)` and `estimated_tokens` is `Some(est)` and `est > limit as usize`, skip with `continue`
- [x] 5.2 Update the SQL query in `proxy.rs` that fetches `GroupServerDetail` rows to include `max_input_tokens` ← (verify: a server with max_input_tokens=100 is skipped for a large request; a server with max_input_tokens=NULL is never skipped; estimated_tokens=None never causes a skip; `cargo clippy -- -D warnings` passes)

## 6. Frontend Store

- [x] 6.1 Add `max_input_tokens: number | null` to the `GroupServerDetail` interface in `src/stores/groups.ts`
- [x] 6.2 Add `max_input_tokens?: number | null` to the `updateAssignment` function's input type in `src/stores/groups.ts` ← (verify: TypeScript strict mode passes with `bun run build` or `just check`)

## 7. Frontend UI — Edit Dialog

- [x] 7.1 In `src/pages/GroupDetailPage.vue`, add `editServerTokenForm` reactive ref: `ref({ max_input_tokens: null as number | null })`
- [x] 7.2 Populate `editServerTokenForm` when the edit dialog opens (set from the server's current `max_input_tokens` value)
- [x] 7.3 Add a "Max Input Tokens" number input field in the Edit Server dialog, following the same layout pattern as the Rate Limit section (label + q-input with type="number", clearable, hint text noting it is approximate)
- [x] 7.4 Include `max_input_tokens: editServerTokenForm.value.max_input_tokens` in the `updateAssignment` call in the save handler ← (verify: editing a server and setting max_input_tokens saves correctly; clearing the field sends null; dialog opens with existing value pre-filled)

## 8. Frontend UI — Server List Badge

- [x] 8.1 In the server list in `GroupDetailPage.vue`, add a badge that displays when `server.max_input_tokens` is non-null, showing a label such as "≤Xk tokens" (format the number for readability, e.g., 30000 → "≤30K tokens") ← (verify: badge appears for servers with a configured threshold; badge is absent for servers with max_input_tokens=null; formatting looks correct in the UI)

## 9. Final Check

- [x] 9.1 Run `just check` and fix all lint and type errors in both frontend and backend ← (verify: `just check` exits with code 0, no warnings or errors from clippy or biome)
