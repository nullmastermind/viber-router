## 1. Database Migrations

- [x] 1.1 Create migration: add `model_request_costs JSONB NOT NULL DEFAULT '{}'` to `subscription_plans` table
- [x] 1.2 Create migration: update `sub_type` CHECK constraint on `subscription_plans` to include 'pay_per_request'
- [x] 1.3 Create migration: add `model_request_costs JSONB NOT NULL DEFAULT '{}'` to `key_subscriptions` table
- [x] 1.4 Create migration: update `sub_type` CHECK constraint on `key_subscriptions` to include 'pay_per_request' ← (verify: both migrations run cleanly with `sqlx migrate run`, existing rows unaffected, CHECK constraints reject invalid sub_type values)

## 2. Rust Models

- [x] 2.1 Add `model_request_costs: serde_json::Value` field to `SubscriptionPlan` struct in `src/models/subscription_plan.rs`
- [x] 2.2 Add `model_request_costs: serde_json::Value` field to `CreateSubscriptionPlan` and `UpdateSubscriptionPlan` structs
- [x] 2.3 Add `model_request_costs: serde_json::Value` field to `KeySubscription` struct in `src/models/key_subscription.rs` ← (verify: `cargo check` passes, all model structs compile with new field)

## 3. Admin Route: Subscription Plans

- [x] 3.1 Accept `pay_per_request` as valid `sub_type` in the create handler in `src/routes/admin/subscription_plans.rs`
- [x] 3.2 Add validation: if `sub_type == "pay_per_request"`, `model_request_costs` must not be empty (return 400 otherwise)
- [x] 3.3 Accept optional `reset_hours` for `pay_per_request` (do not require it)
- [x] 3.4 Include `model_request_costs` in the INSERT query for plan creation
- [x] 3.5 Include `model_request_costs` in the UPDATE query for plan updates ← (verify: POST with pay_per_request type creates plan, POST with empty model_request_costs returns 400, PATCH updates model_request_costs correctly)

## 4. Admin Route: Key Subscriptions

- [x] 4.1 Include `model_request_costs` in the snapshot INSERT query in `src/routes/admin/key_subscriptions.rs` (copy from plan when assigning)
- [x] 4.2 Include `model_request_costs` in the bulk create snapshot INSERT in `src/routes/admin/group_keys.rs` ← (verify: assigning a pay_per_request plan to a sub-key snapshots model_request_costs into key_subscriptions record)

## 5. Subscription Cost Tracking (subscription.rs)

- [x] 5.1 Change windowing condition in `get_total_cost()` from `sub_type == "hourly_reset"` to `reset_hours.is_some()`
- [x] 5.2 Change windowing condition in `get_model_cost()` from `sub_type == "hourly_reset"` to `reset_hours.is_some()`
- [x] 5.3 Change windowing condition in `rebuild_total_cost()` from `sub_type == "hourly_reset"` to `reset_hours.is_some()`
- [x] 5.4 Change windowing condition in `rebuild_model_cost()` from `sub_type == "hourly_reset"` to `reset_hours.is_some()`
- [x] 5.5 Change windowing condition in `update_cost_counters()` from `sub_type == "hourly_reset"` to `reset_hours.is_some()` ← (verify: existing hourly_reset subscriptions still use windowed keys, pay_per_request with reset_hours also uses windowed keys, pay_per_request without reset_hours uses flat keys)

## 6. Subscription Enforcement (subscription.rs)

- [x] 6.1 Add `pay_per_request` to the subscription type ordering in `check_subscriptions()` (after hourly_reset, before fixed)
- [x] 6.2 Add model eligibility check: if `sub_type == "pay_per_request"` and model is not a key in `model_request_costs`, skip this subscription ← (verify: pay_per_request subscription is skipped when model not in model_request_costs, request blocked with 429 when no other subscription covers it)

## 7. Proxy Cost Calculation (proxy.rs)

- [x] 7.1 In the non-streaming response path, if active subscription is `pay_per_request`, set cost to `model_request_costs[model]` instead of calling `calculate_cost()`
- [x] 7.2 In the streaming response path, apply the same flat cost logic for `pay_per_request` subscriptions ← (verify: pay_per_request subscription is charged flat cost, not token-based cost, in both streaming and non-streaming paths; cargo clippy passes with -D warnings)

## 8. Frontend: Subscription Type Composable

- [x] 8.1 Add `pay_per_request` entry to `src/composables/useSubscriptionType.ts` with label "Pay Per Request" and tooltip "Each request costs a flat rate based on the model. Budget can optionally reset."

## 9. Frontend: Plans Page

- [x] 9.1 Add `model_request_costs` column to the plans table in `src/pages/PlansPage.vue`, showing chips (e.g., "claude-sonnet-4-6: $0.10"), visible only for pay_per_request plans
- [x] 9.2 Add `pay_per_request` option to the sub_type select in the create/edit dialog
- [x] 9.3 Show `reset_hours` field when sub_type is `pay_per_request` OR `hourly_reset` (update existing conditional)
- [x] 9.4 Add model request costs editor (model selector + $ per request input) to the dialog, visible only when sub_type is `pay_per_request`
- [x] 9.5 Add frontend validation: if sub_type is `pay_per_request` and model_request_costs is empty, show error and block submit
- [x] 9.6 Include `model_request_costs` in the save payload for create and update ← (verify: creating a pay_per_request plan via UI sends correct payload, model_request_costs editor shows/hides correctly, reset_hours field shows for both hourly_reset and pay_per_request)

## 10. Final Check

- [x] 10.1 Run `just check` and fix all type errors and lint errors in both frontend and backend ← (verify: `just check` exits 0 with no errors or warnings)
