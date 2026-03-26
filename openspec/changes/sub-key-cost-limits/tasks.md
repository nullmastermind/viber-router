## 1. Database Migrations

- [x] 1.1 Create migration for `subscription_plans` table (id, name, sub_type, cost_limit_usd, model_limits JSONB, reset_hours, duration_days, is_active, created_at, updated_at)
- [x] 1.2 Create migration for `key_subscriptions` table (id, group_key_id FK, plan_id FK, sub_type, cost_limit_usd, model_limits, reset_hours, duration_days, status, activated_at, expires_at, created_at) with index on (group_key_id, status)
- [x] 1.3 Create migration to add `cost_usd FLOAT8` and `subscription_id UUID` columns to `token_usage_logs`
- [x] 1.4 Create migration to drop `monthly_token_limit` and `monthly_request_limit` from `group_keys` ← (verify: all 4 migrations run without errors, schema matches design.md)

## 2. Rust Models

- [x] 2.1 Create `src/models/subscription_plan.rs` — SubscriptionPlan, CreateSubscriptionPlan, UpdateSubscriptionPlan structs with sqlx derives
- [x] 2.2 Create `src/models/key_subscription.rs` — KeySubscription struct with sqlx derives
- [x] 2.3 Update `src/models/group_key.rs` — remove `monthly_token_limit` and `monthly_request_limit` from GroupKey struct
- [x] 2.4 Update `src/models/mod.rs` — add new module exports
- [x] 2.5 Update `src/usage_buffer.rs` — add `cost_usd: Option<f64>` and `subscription_id: Option<Uuid>` to TokenUsageEntry, update flush_batch INSERT query ← (verify: TokenUsageEntry has new fields, flush_batch includes cost_usd and subscription_id in UNNEST)

## 3. Model Pricing Cache

- [x] 3.1 Add pricing cache type to AppState: `Arc<RwLock<HashMap<String, ModelPricing>>>` with ModelPricing struct (input_1m_usd, output_1m_usd, cache_write_1m_usd, cache_read_1m_usd)
- [x] 3.2 Create pricing cache refresh function — query all models, update the RwLock map
- [x] 3.3 Spawn background task in main.rs to refresh pricing cache every 60 seconds (follow partition maintenance pattern)
- [x] 3.4 Add server rate data to GroupConfig or make it available during cost calculation ← (verify: pricing cache loads on startup, refreshes every 60s, AppState compiles with new field)

## 4. Subscription Plans Admin API

- [x] 4.1 Create `src/routes/admin/subscription_plans.rs` — router with CRUD endpoints (list, create, update, delete)
- [x] 4.2 Implement POST create — validate sub_type, require reset_hours for hourly_reset
- [x] 4.3 Implement GET list — return all plans ordered by created_at desc
- [x] 4.4 Implement PATCH update — partial update of name, sub_type, cost_limit_usd, model_limits, reset_hours, duration_days, is_active
- [x] 4.5 Implement DELETE — check for existing subscriptions, return 409 if referenced
- [x] 4.6 Register routes in `src/routes/admin/mod.rs` at `/subscription-plans` ← (verify: all CRUD endpoints respond correctly, validation rejects invalid input)

## 5. Key Subscriptions Admin API

- [x] 5.1 Create `src/routes/admin/key_subscriptions.rs` — router with assign, list, cancel endpoints
- [x] 5.2 Implement POST assign — lookup plan, validate is_active, snapshot values into key_subscriptions
- [x] 5.3 Implement GET list — return all subscriptions for a key ordered by created_at desc
- [x] 5.4 Implement PATCH cancel — validate status is 'active', set to 'cancelled', invalidate Redis cache `key_subs:{group_key_id}`
- [x] 5.5 Register routes nested under group_keys router at `/{key_id}/subscriptions` ← (verify: assign snapshots plan values correctly, cancel invalidates Redis cache)

## 6. Subscription Enforcement in Proxy

- [x] 6.1 Create `src/subscription.rs` module — subscription check logic: load subscriptions from Redis/DB, sort by priority (hourly_reset first, then fixed, FIFO), check budgets
- [x] 6.2 Implement Redis cost counter read — GET total, per-model, and window counters with pipeline/MGET
- [x] 6.3 Implement Redis rebuild on cache miss — query SUM(cost_usd) from token_usage_logs
- [x] 6.4 Implement subscription list cache — GET/SET `key_subs:{group_key_id}` with DB fallback
- [x] 6.5 Implement lazy activation — SETNX `sub_activated:{sub_id}`, update DB on first charge
- [x] 6.6 Implement expiration check — compare now vs expires_at, mark expired in DB
- [x] 6.7 Implement exhaustion check — compare total cost vs cost_limit_usd, mark exhausted in DB
- [x] 6.8 Integrate into proxy.rs — add subscription check after resolve_group_config, before forwarding request. Return 429 if all subs blocked ← (verify: proxy returns 429 when all subs exhausted, allows unlimited keys, respects priority order hourly>fixed)

## 7. Cost Calculation and Redis Update in Proxy

- [x] 7.1 Create cost calculation function — lookup model pricing from cache, lookup server rates, compute cost_usd
- [x] 7.2 Update non-streaming path (proxy.rs ~line 863) — calculate cost, update Redis counters (INCRBYFLOAT total + per-model), include cost_usd and subscription_id in TokenUsageEntry
- [x] 7.3 Update streaming path (UsageTrackingStream) — add subscription_id and pricing cache to struct, calculate cost and update Redis on stream completion
- [x] 7.4 Handle model pricing not found — set cost_usd = None, subscription_id = None, skip Redis update ← (verify: Redis counters increment correctly for both streaming and non-streaming, cost formula matches design.md including server rates)

## 8. Frontend — Plans Page

- [x] 8.1 Create `src/pages/PlansPage.vue` — table with columns: Name, Type, Cost Limit, Model Limits, Reset Hours, Duration, Active toggle
- [x] 8.2 Add create plan dialog with form fields: name, type select, cost limit, duration, reset hours (conditional), model limits editor (model dropdown from models store + dollar input)
- [x] 8.3 Add edit plan functionality
- [x] 8.4 Add delete plan with confirmation
- [x] 8.5 Add route in `src/router/routes.ts` and sidebar link in `src/layouts/MainLayout.vue` ← (verify: Plans page loads, CRUD works, model limits editor uses models from DB)

## 9. Frontend — Key Subscriptions UI

- [x] 9.1 Add subscription types and API methods to `src/stores/groups.ts` — KeySubscription interface, fetchKeySubscriptions, assignSubscription, cancelSubscription
- [x] 9.2 Add subscriptions section in GroupDetailPage expanded sub-key row — positioned before SubKeyUsage, shows subscription table with Plan, Type, Budget, Used, Status columns
- [x] 9.3 Add "Add Subscription" button with plan dropdown (fetched from plans API, filtered to is_active)
- [x] 9.4 Add cancel button for active subscriptions
- [x] 9.5 Add status badges (green=active, red=exhausted, orange=expired, grey=cancelled)
- [x] 9.6 Update GroupKey interface — remove monthly_token_limit and monthly_request_limit ← (verify: subscriptions display in expanded row, add/cancel work, status badges render correctly)

## 10. Cleanup and Verification

- [x] 10.1 Run `just check` — fix all type errors and lint errors
- [x] 10.2 Run `cargo test` — ensure existing tests pass, update usage_buffer tests for new TokenUsageEntry fields ← (verify: `just check` passes clean, `cargo test` passes, no regressions)
