## Why

The existing subscription system only supports token-based billing (`fixed`, `hourly_reset`), which doesn't fit use cases where admins want to charge a flat fee per API call regardless of token count. Adding `pay_per_request` enables per-model flat-rate billing, giving admins a simpler pricing model for predictable cost control.

## What Changes

- Add a new `pay_per_request` subscription type alongside `fixed` and `hourly_reset`
- Add `model_request_costs` JSONB field to `subscription_plans` and `key_subscriptions` tables to store per-model flat costs
- Block requests to models not listed in `model_request_costs` when the active subscription is `pay_per_request`
- Support optional `reset_hours` for `pay_per_request` (budget can reset periodically or be one-time)
- Change windowing logic from checking `sub_type == "hourly_reset"` to `reset_hours.is_some()` so `pay_per_request` with a reset window works correctly
- Apply flat cost (not token-based cost) in post-response billing when subscription type is `pay_per_request`
- Snapshot `model_request_costs` from plan when assigning subscriptions to keys
- Add frontend UI for configuring `model_request_costs` in the Plans page

## Capabilities

### New Capabilities

- `pay-per-request-billing`: Flat-rate per-model billing for the `pay_per_request` subscription type, including request blocking for unlisted models and post-response cost deduction

### Modified Capabilities

- `subscription-plans`: New `sub_type` value and new `model_request_costs` field added to plan schema
- `subscription-enforcement`: Request blocking and cost calculation logic extended for `pay_per_request`
- `subscription-cost-tracking`: Windowing logic generalized from `sub_type == "hourly_reset"` to `reset_hours.is_some()`
- `subscription-plans-ui`: Plans page gains `model_request_costs` editor and updated sub_type options
- `key-subscriptions`: Snapshot includes new `model_request_costs` field

## Impact

- Database: 2 migrations (add column + update CHECK constraints on both tables)
- Rust backend: `subscription_plan.rs`, `key_subscription.rs`, `subscription_plans.rs` route, `key_subscriptions.rs` route, `group_keys.rs` route, `subscription.rs`, `proxy.rs`
- Frontend: `useSubscriptionType.ts`, `PlansPage.vue`, `GroupDetailPage.vue`
- No breaking changes to existing `fixed` or `hourly_reset` subscriptions
