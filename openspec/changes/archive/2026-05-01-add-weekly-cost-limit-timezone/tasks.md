## 1. Database and Backend Data Model

- [x] 1.1 Add migration 045 for `settings.timezone`, `subscription_plans.weekly_cost_limit_usd`, and `key_subscriptions.weekly_cost_limit_usd` with the timezone default set to `Asia/Ho_Chi_Minh`
- [x] 1.2 Add `timezone` to settings models and settings admin request/response structs
- [x] 1.3 Add `weekly_cost_limit_usd: Option<f64>` to subscription plan create/update/read models
- [x] 1.4 Add `weekly_cost_limit_usd: Option<f64>` to key subscription models and SQL row mappings ← (verify: all affected SELECT/INSERT/UPDATE SQL includes the new columns and NULL weekly limits remain unlimited)

## 2. Settings and Cache

- [x] 2.1 Update admin settings GET/PUT/default settings SQL to load, save, and return `timezone`
- [x] 2.2 Validate or defensively handle IANA timezone values, falling back to `Asia/Ho_Chi_Minh` for runtime calculations if stored data is invalid
- [x] 2.3 Add Redis cache helpers for `get_timezone()`, `set_timezone()`, and `invalidate_timezone()` using the existing settings cache pattern
- [x] 2.4 Invalidate `settings:timezone` when admin settings updates include `timezone` ← (verify: timezone changes are reflected by subsequent subscription checks without restarting the API)

## 3. Subscription Plan and Assignment APIs

- [x] 3.1 Accept and return `weekly_cost_limit_usd` in subscription plan create and list/read routes
- [x] 3.2 Accept `weekly_cost_limit_usd` in subscription plan update routes, including clearing the value to NULL
- [x] 3.3 Snapshot `weekly_cost_limit_usd` from a plan into `key_subscriptions` when assigning a plan to a key
- [x] 3.4 Preserve existing bonus subscription creation behavior without requiring weekly limit fields ← (verify: plan assignment copies weekly limits, bonus assignment still succeeds with NULL weekly fields)

## 4. Weekly Cost Enforcement

- [x] 4.1 Add a timezone-aware helper that returns current Monday start UTC, next Monday start UTC, Monday epoch, TTL seconds, and reset timestamp for the configured timezone
- [x] 4.2 Add `get_weekly_cost(state, sub)` that reads Redis key `sub_weekly_cost:{sub_id}:w:{monday_epoch}` and rebuilds from `token_usage_logs` on cache miss
- [x] 4.3 Update `check_subscriptions()` to skip non-bonus subscriptions whose weekly cost is greater than or equal to `weekly_cost_limit_usd`, after existing total budget checks and without marking the subscription exhausted
- [x] 4.4 Update `update_cost_counters()` to increment the weekly Redis counter and set TTL until next Monday for non-bonus subscriptions with a weekly limit
- [x] 4.5 Keep weekly tracking out of bonus subscription flows and preserve existing lifetime, hourly reset, RPM, and TPM behavior ← (verify: weekly-limit hit falls through to next subscription and weekly counter rebuild matches DB sum for the configured calendar week)

## 5. Public Usage API

- [x] 5.1 Add `weekly_cost_used`, `weekly_cost_limit_usd`, and `weekly_reset_at` to the public subscription response model
- [x] 5.2 Populate weekly usage fields for non-bonus subscriptions with weekly limits using the same timezone and Redis/DB helper semantics as enforcement
- [x] 5.3 Return null or non-limiting weekly fields for subscriptions with `weekly_cost_limit_usd = NULL` and keep bonus subscription response shape unchanged ← (verify: public response includes weekly fields for limited subscriptions and does not expose provider/server details)

## 6. Frontend Admin UI

- [x] 6.1 Add a General card with timezone select options to `SettingsPage.vue`, defaulting to `Asia/Ho_Chi_Minh` and saving through the settings API
- [x] 6.2 Add optional Weekly Cost Limit field to the create/edit plan dialog in `PlansPage.vue`
- [x] 6.3 Add Weekly Cost Limit display to the plans list table
- [x] 6.4 Add Weekly Cost Limit display to key subscription tables in `GroupDetailPage.vue` ← (verify: empty weekly limit displays as unlimited/blank and numeric limits round/display consistently as USD)

## 7. Frontend Public Usage UI

- [x] 7.1 Update public usage TypeScript types to include weekly usage, weekly limit, and weekly reset fields
- [x] 7.2 Display weekly cost progress and reset timing on non-bonus subscription cards when `weekly_cost_limit_usd` is present
- [x] 7.3 Hide weekly progress for unlimited weekly subscriptions and keep bonus card rendering unchanged ← (verify: protected attribution links in `PublicUsagePage.vue` remain unchanged while weekly data renders correctly)

## 8. Validation and Checks

- [x] 8.1 Add or update backend tests/manual verification coverage for timezone week boundary calculation, Redis miss rebuild, and subscription skip behavior
- [x] 8.2 Add or update frontend validation where needed for optional numeric weekly limit and timezone selection
- [x] 8.3 Run `just check` from `/root/projects/viber-router` and fix all reported issues ← (verify: cargo check, clippy with warnings denied, frontend type/lint checks all pass)
