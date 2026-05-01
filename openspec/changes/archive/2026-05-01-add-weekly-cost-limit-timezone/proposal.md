## Why

Subscription owners need a weekly spending guardrail that resets on business calendar boundaries rather than only relying on lifetime budgets or request/window limits. Because Viber Router operates for Vietnamese admins by default, the week boundary must be derived from a configurable global timezone with `Asia/Ho_Chi_Minh` as the default.

## What Changes

- Add a global timezone setting, defaulting to `Asia/Ho_Chi_Minh`, used for calendar week boundary calculations.
- Add optional `weekly_cost_limit_usd` fields to subscription plans and assigned key subscriptions.
- Track per-subscription weekly spend for non-bonus subscriptions using Redis counters with database rebuild fallback from token usage logs.
- Skip a subscription once its weekly cost reaches the limit and continue evaluating later subscriptions without marking the skipped subscription exhausted.
- Expose weekly usage, weekly limit, and weekly reset time in public usage responses and UI.
- Add admin UI controls for timezone and weekly subscription plan limits, and show weekly limits in group subscription tables.
- No changes to bonus subscriptions or existing lifetime cost, reset-hours, RPM, or TPM behavior.

## Capabilities

### New Capabilities
- `subscription-weekly-cost-limit`: Calendar-week cost limits for non-bonus subscriptions, including enforcement, counters, and public usage reporting.

### Modified Capabilities
- `config-cache`: Global settings cache includes timezone so runtime enforcement can calculate calendar week boundaries consistently.
- `subscription-plans`: Subscription plans can define an optional weekly cost limit copied into assigned subscriptions.
- `subscription-plans-ui`: Admin plan forms expose the optional weekly cost limit.
- `key-subscriptions`: Assigned key subscriptions store the weekly cost limit snapshot from their plan.
- `subscription-enforcement`: Subscription selection skips non-bonus subscriptions whose weekly cost has reached the configured weekly limit.
- `public-usage-api`: Public usage responses include weekly cost usage, weekly limit, and weekly reset time.
- `public-usage-page`: Public usage UI displays weekly cost usage and reset information.
- `admin-ui`: Settings UI includes a general timezone selector.

## Impact

- Database migration adds `settings.timezone`, `subscription_plans.weekly_cost_limit_usd`, and `key_subscriptions.weekly_cost_limit_usd`.
- Backend models, admin routes, public usage routes, subscription enforcement, and Redis cache helpers are updated.
- Redis adds weekly cost counter keys in the form `sub_weekly_cost:{sub_id}:w:{monday_epoch}` with TTL until the next configured Monday 00:00.
- Frontend settings, plans, group detail, and public usage pages are updated.
- Requires date/timezone handling in Rust for IANA timezone names and validation/fallback behavior.
- `just check` must pass after implementation.
