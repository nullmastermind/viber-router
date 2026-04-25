## 1. Database Migration

- [x] 1.1 Create migration file `viber-router-api/migrations/035_add_bonus_subscription_columns.sql`
- [x] 1.2 Add five nullable columns to `key_subscriptions`: `bonus_base_url TEXT`, `bonus_api_key TEXT`, `bonus_name TEXT`, `bonus_quota_url TEXT`, `bonus_quota_headers JSONB`
- [x] 1.3 Drop and recreate `sub_type` CHECK constraint on `key_subscriptions` to include `'bonus'`
- [x] 1.4 Drop and recreate `sub_type` CHECK constraint on `subscription_plans` to include `'bonus'` ← (verify: migration runs cleanly with `sqlx migrate run`, both tables accept `sub_type = 'bonus'`, non-bonus rows unaffected)

## 2. Backend Model

- [x] 2.1 Add five optional fields to `KeySubscription` struct in `viber-router-api/src/models/key_subscription.rs`: `bonus_base_url: Option<String>`, `bonus_api_key: Option<String>`, `bonus_name: Option<String>`, `bonus_quota_url: Option<String>`, `bonus_quota_headers: Option<serde_json::Value>`
- [x] 2.2 Update all `sqlx` queries that SELECT from `key_subscriptions` to include the new columns ← (verify: `cargo check` passes, all query result mappings compile without error)

## 3. Subscription Engine

- [x] 3.1 Add `BonusServer` struct to `viber-router-api/src/subscription.rs` with fields: `subscription_id: Uuid`, `base_url: String`, `api_key: String`, `name: String`
- [x] 3.2 Add `BonusServers { servers: Vec<BonusServer>, fallback_subscription: Option<(Uuid, Option<f64>)> }` variant to `SubCheckResult` enum
- [x] 3.3 Update `check_subscriptions()` to separate bonus subs from non-bonus subs
- [x] 3.4 Run existing non-bonus logic on non-bonus subs to derive `fallback_subscription`
- [x] 3.5 Collect active bonus subs sorted by `created_at ASC`; if non-empty return `BonusServers`; otherwise return existing result unchanged
- [x] 3.6 Fix all exhaustive match arms on `SubCheckResult` in the rest of the codebase to handle the new `BonusServers` variant ← (verify: `cargo clippy -- -D warnings` passes, `BonusServers` is handled in every match)

## 4. Admin API — Bonus Creation Endpoint

- [x] 4.1 Add `CreateBonusSubscriptionRequest` struct (or extend existing request struct) in `viber-router-api/src/routes/admin/key_subscriptions.rs` with fields: `bonus_name`, `bonus_base_url`, `bonus_api_key`, `bonus_quota_url`, `bonus_quota_headers`
- [x] 4.2 Extend the POST handler to detect bonus creation path (no `plan_id`, bonus fields present) and validate required fields (`bonus_name`, `bonus_base_url`, `bonus_api_key`)
- [x] 4.3 Insert bonus subscription with hardcoded values: `sub_type = 'bonus'`, `cost_limit_usd = 0`, `model_limits = {}`, `model_request_costs = {}`, `duration_days = 36500`, all other budget fields NULL ← (verify: POST with bonus fields returns 201 with correct record; POST missing required bonus fields returns 400; POST with neither plan_id nor bonus fields returns 400)

## 5. Proxy Engine — Bonus Waterfall

- [x] 5.1 In `viber-router-api/src/routes/proxy.rs`, after subscription check, add match arm for `SubCheckResult::BonusServers`
- [x] 5.2 Implement bonus server iteration loop: build upstream URL (`bonus_base_url.trim_end_matches('/') + "/v1/messages"`), set `x-api-key` and `authorization: Bearer` headers, forward transformed request body
- [x] 5.3 On 2xx from bonus server: log token usage with `subscription_id = bonus_sub_id` and `cost_usd = NULL`, return response immediately (handling both streaming and non-streaming paths)
- [x] 5.4 On non-2xx from bonus server: log failure, continue to next bonus server
- [x] 5.5 After all bonus servers exhausted: if `fallback_subscription` is `Some`, set `selected_subscription_id` and proceed to group server waterfall; if `None`, proceed to group server waterfall without subscription tracking ← (verify: end-to-end proxy flow — bonus 2xx returns immediately, bonus all-fail with fallback charges correct sub, bonus all-fail with no fallback reaches group servers)

## 6. Public Usage API — Bonus Fields

- [x] 6.1 Add `bonus_name: Option<String>`, `bonus_quotas: Option<Vec<QuotaInfo>>`, `bonus_usage: Option<Vec<BonusModelUsage>>` fields to `PublicSubscription` struct in `viber-router-api/src/routes/public/usage.rs`
- [x] 6.2 Add `QuotaInfo` struct: `name: String`, `utilization: f64`, `reset_at: Option<String>`, `description: Option<String>`
- [x] 6.3 Add `BonusModelUsage` struct: `model: String`, `request_count: i64`
- [x] 6.4 For each bonus subscription in the response: query `token_usage_logs` for per-model request counts in the last 30 days where `subscription_id` matches
- [x] 6.5 For each bonus subscription with `bonus_quota_url` set: make HTTP GET with `bonus_quota_headers` (if set) and 5-second timeout; parse `{ "quotas": [...] }` response; return empty array on any error ← (verify: API response includes `bonus_quotas` and `bonus_usage` for bonus subs; quota URL timeout returns empty array without failing; `cargo clippy -- -D warnings` passes)

## 7. useSubscriptionType Composable

- [x] 7.1 Add `'bonus'` case to `getSubTypeLabel()` returning `"Bonus"` in `src/composables/useSubscriptionType.ts`
- [x] 7.2 Add `'bonus'` case to `getSubTypeTooltip()` returning `"External API subscription with its own server. Tried before group servers."`
- [x] 7.3 Add `{ value: 'bonus', label: 'Bonus' }` option to `getSubTypeOptions()` ← (verify: all three functions handle 'bonus' without TypeScript errors; `bun run lint` passes)

## 8. Admin UI — GroupDetailPage

- [x] 8.1 Add "Add Bonus" button alongside the existing "Add Subscription" button in the sub-key expanded row subscriptions section
- [x] 8.2 Create "Add Bonus" dialog component (inline or separate) with fields: Name (required), Base URL (required, default "https://api.anthropic.com"), API Key (required), Quota Check URL (optional), Quota Headers (optional JSON text)
- [x] 8.3 Wire dialog submit to POST bonus payload to `/api/admin/groups/:group_id/keys/:key_id/subscriptions` and refresh subscription list on success
- [x] 8.4 In the subscription table, render bonus rows: show `bonus_name` in Plan column, "Bonus" in Type column, "N/A" in Budget and Used columns ← (verify: "Add Bonus" button opens dialog; successful submission adds row to table; bonus rows display correctly; `bun run lint` passes)

## 9. Public Usage UI — PublicUsagePage

- [x] 9.1 In `src/pages/PublicUsagePage.vue`, detect and separate bonus subscriptions from non-bonus in the subscription list
- [x] 9.2 Render bonus subscription card with: `bonus_name` as title + lightning bolt icon, quota section (q-linear-progress bars per quota entry with utilization %, name label, reset countdown if `reset_at` set), `bonus_usage` list of model/count pairs
- [x] 9.3 Handle `bonus_quotas` null (omit quota section), empty array (show "Quota info unavailable"), non-empty (render progress bars)
- [x] 9.4 Ensure non-bonus subscription cards render unchanged ← (verify: bonus card renders correctly for all quota states; non-bonus cards unaffected; `bun run lint` passes)

## 10. Integration Check

- [x] 10.1 Run `just check` (type-check + lint for both frontend and backend) and fix all errors ← (verify: `just check` exits 0 with no warnings or errors)
