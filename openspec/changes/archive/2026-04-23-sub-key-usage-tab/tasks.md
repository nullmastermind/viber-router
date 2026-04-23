## 1. Backend — By-Key Usage Endpoint

- [x] 1.1 Add `KeyTokenUsage` response struct in `viber-router-api/src/routes/admin/token_usage.rs` with fields: `group_key_id` (Option<Uuid>), `key_name` (Option<String>), `created_at` (Option<DateTime>), `total_input_tokens` (i64), `total_output_tokens` (i64), `total_cache_creation_tokens` (i64), `total_cache_read_tokens` (i64), `request_count` (i64), `cost_usd` (Option<f64>)
- [x] 1.2 Add `KeyUsageResponse` struct wrapping `Vec<KeyTokenUsage>` as `keys` field
- [x] 1.3 Implement `get_token_usage_by_key` handler: accept `TokenUsageParams` (reuse existing struct — only `group_id`, `period`, `start`, `end` are used), build SQL query that aggregates `token_usage_logs` grouped by `group_key_id`, LEFT JOIN `group_keys` for `name`/`created_at`, LEFT JOIN `models`+`group_servers` for cost calculation (same CASE expression as existing handler), ORDER BY `request_count DESC`
- [x] 1.4 Handle both period-shortcut and absolute date-range branches (reuse `resolve_interval` for period)
- [x] 1.5 Register the new handler: update `pub fn router()` in `token_usage.rs` to add `.route("/by-key", get(get_token_usage_by_key))`
- [x] 1.6 Run `cargo check` and `cargo clippy -- -D warnings` to verify the backend compiles cleanly <- (verify: endpoint compiles, route is registered under /api/admin/token-usage/by-key, response shape matches spec)

## 2. Frontend — Store Layer

- [x] 2.1 Add `KeyTokenUsage` interface in `src/stores/groups.ts` with fields matching the backend response: `group_key_id` (string | null), `key_name` (string | null), `created_at` (string | null), `total_input_tokens` (number), `total_output_tokens` (number), `total_cache_creation_tokens` (number), `total_cache_read_tokens` (number), `request_count` (number), `cost_usd` (number | null)
- [x] 2.2 Add `KeyUsageResponse` interface: `{ keys: KeyTokenUsage[] }`
- [x] 2.3 Add `fetchTokenUsageByKey(groupId: string, params?: { start?: string; end?: string; period?: string })` action that calls `GET /api/admin/token-usage/by-key` and returns `KeyUsageResponse`
- [x] 2.4 Export the new action from the store's return object <- (verify: types match spec, action is exported and callable)

## 3. Frontend — Child Tabs Structure

- [x] 3.1 In `GroupDetailPage.vue`, wrap the existing Token Usage tab panel content with child `q-tabs` (dense, inline-label) containing two `q-tab` items: "By Server" (value `by-server`) and "By Sub-Key" (value `by-sub-key`). Add a reactive `tokenUsageChildTab` ref defaulting to `'by-server'`
- [x] 3.2 Wrap the existing token usage content (period toggle, filters, table) inside a `q-tab-panel` with name `by-server` under the new child `q-tab-panels`
- [x] 3.3 Add a `q-tab-panel` with name `by-sub-key` as placeholder (empty div) for the next task group
- [x] 3.4 Verify "By Server" tab renders identically to before — no visual or behavioral changes <- (verify: existing token usage UI unchanged, child tabs appear and switch correctly)

## 4. Frontend — By Sub-Key Tab Content

- [x] 4.1 Add reactive state for the By Sub-Key tab: `subKeyUsageData` (ref<KeyTokenUsage[]>), `subKeyUsageLoading` (boolean), `subKeyUsageError` (string), `subKeyDateStart` (string, default 30 days ago ISO), `subKeyDateEnd` (string, default now ISO)
- [x] 4.2 Implement `loadSubKeyUsage()` function: calls `groupsStore.fetchTokenUsageByKey(groupId, { start, end })`, populates `subKeyUsageData`, handles loading/error states
- [x] 4.3 Add date range filter UI in the by-sub-key tab panel: two `q-input` elements with `type="date"` for start and end, plus a refresh button that calls `loadSubKeyUsage()`
- [x] 4.4 Define `subKeyUsageColumns` array with columns: Key Name (left-aligned, format null as "Master / Dynamic Keys" or "Deleted Key"), Input Tokens (right, sortable, formatCompact), Output Tokens (right, sortable, formatCompact), Cache Write (right, sortable, formatCompact), Cache Read (right, sortable, formatCompact), Requests (right, sortable, formatCompact), Cost (right, sortable, format as $X.XXXX or em dash), Created At (left, format as locale date or em dash)
- [x] 4.5 Add `q-table` rendering `subKeyUsageData` with `subKeyUsageColumns`, row-key based on `group_key_id`, sortable columns, and hide-pagination with `rowsPerPage: 0`
- [x] 4.6 Add loading state (q-spinner), error state (q-banner with retry), and empty state (q-banner "No usage data for this period") following the same patterns as the By Server tab
- [x] 4.7 Add total row in `#bottom-row` slot summing all numeric columns (input, output, cache write, cache read, requests, cost) <- (verify: table renders with all columns, sorting works, states display correctly, totals are accurate)

## 5. Frontend — Expandable Subscription Rows

- [x] 5.1 Add expand toggle to sub-key usage table rows: use `q-table`'s expand functionality, only for rows where `group_key_id` is not null
- [x] 5.2 In the expanded row template, call `loadKeySubscriptions(row.group_key_id)` (reuse existing function from Keys tab) and display subscriptions using existing `subColumns` and subscription table pattern
- [x] 5.3 Handle the master/dynamic key row (null `group_key_id`): do not show expand icon or allow expansion <- (verify: clicking a sub-key row expands to show subscriptions, master key row has no expand, subscription data loads correctly)

## 6. Verification

- [x] 6.1 Run `just check` to verify both frontend and backend pass type-check and lint <- (verify: zero errors from type-check and lint across both frontend and backend)
