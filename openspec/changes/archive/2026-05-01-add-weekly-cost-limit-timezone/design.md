## Context

Viber Router already enforces subscription budgets and rate limits for non-bonus subscriptions using PostgreSQL as the source of truth and Redis counters for hot-path checks. Current limits are lifetime-style (`cost_limit_usd`) or window/request/token based (`reset_hours`, RPM, TPM), but there is no calendar-week spending guardrail. Settings already include globally managed values and Redis-backed cache patterns, such as blocked paths, that can be extended for timezone lookup.

The new weekly limit spans database schema, admin APIs, runtime subscription selection, usage reporting, and frontend display. The calendar week must be Monday 00:00 through the next Monday 00:00 in a configurable IANA timezone, defaulting to `Asia/Ho_Chi_Minh`.

## Goals / Non-Goals

**Goals:**

- Persist a global timezone setting and cache it for runtime subscription checks.
- Persist optional weekly cost limits on subscription plans and assigned key subscriptions.
- Enforce weekly limits for fixed, hourly_reset, and pay_per_request subscriptions without marking subscriptions exhausted.
- Maintain Redis-first weekly cost lookup with database rebuild fallback from `token_usage_logs`.
- Expose weekly usage state to admins and public usage viewers.
- Preserve existing lifetime cost, reset-hours, RPM, TPM, and bonus subscription behavior.

**Non-Goals:**

- No weekly cost tracking or enforcement for bonus subscriptions.
- No rolling seven-day windows.
- No changes to billing formulas, token extraction, model pricing, or subscription selection priority except skipping subscriptions that reached the weekly limit.
- No timezone-aware display overhaul beyond the weekly reset information and settings selector.

## Decisions

1. **Use IANA timezone names stored in settings.**
   - Store `settings.timezone TEXT NOT NULL DEFAULT 'Asia/Ho_Chi_Minh'`.
   - Runtime code should validate/parse the setting with the selected Rust timezone library and fall back to the default if existing cached/database data is invalid.
   - Alternative considered: fixed GMT+7 offset. Rejected because the requirement calls for configurable timezone and IANA names avoid future DST/offset mistakes for other regions.

2. **Calendar week boundaries are computed in configured local time and converted to UTC.**
   - The current week starts at Monday 00:00 local time and ends at the next Monday 00:00 local time.
   - Redis keys use the Monday start epoch after conversion to UTC: `sub_weekly_cost:{sub_id}:w:{monday_epoch}`.
   - Alternative considered: UTC weeks. Rejected because admins need local calendar weeks.

3. **Weekly counters are Redis-first with token log rebuild fallback.**
   - `get_weekly_cost(state, sub)` reads the weekly Redis counter.
   - On cache miss, it rebuilds with `SUM(cost_usd)` over `token_usage_logs` where `subscription_id = $1`, `created_at >= monday_start_utc`, and `created_at < next_monday_start_utc`, then writes Redis with TTL until next Monday 00:00 local time.
   - Alternative considered: query PostgreSQL on every subscription check. Rejected because subscription checks run on the proxy path.

4. **Reached weekly limit causes skip, not exhaustion.**
   - During `check_subscriptions()`, after lifetime/total budget checks and before accepting the candidate subscription, compare weekly cost to `weekly_cost_limit_usd`.
   - If `weekly_cost_limit_usd` is `NULL`, skip the weekly check entirely.
   - If `weekly_cost >= weekly_cost_limit_usd`, continue to the next candidate subscription.
   - Do not update subscription status to `exhausted` for weekly-limit hits because the subscription can become eligible again next week.
   - Alternative considered: mark exhausted and reactivate later. Rejected because it complicates status lifecycle and conflicts with existing skip behavior for full hourly windows.

5. **Plan assignment snapshots weekly limits.**
   - Add `weekly_cost_limit_usd` to plan create/update APIs and copy the plan value into `key_subscriptions` when assigning a plan.
   - Existing active subscriptions keep their stored snapshot unless implementation explicitly adds plan sync behavior for updated plans.
   - Alternative considered: lookup plan weekly limit dynamically. Rejected because existing key subscriptions snapshot plan values for enforcement.

6. **Public usage exposes weekly fields only for non-bonus budget subscriptions.**
   - Add `weekly_cost_used`, `weekly_cost_limit_usd`, and `weekly_reset_at` to public subscription objects.
   - For unlimited weekly limit (`NULL`), `weekly_cost_limit_usd` remains null and weekly usage/reset can be null unless the implementation chooses to show current weekly usage for informational purposes.
   - Bonus subscription entries continue using bonus quota/usage fields instead of budget fields.

## Risks / Trade-offs

- **Invalid timezone in settings** -> Validate in admin update when possible and fall back to `Asia/Ho_Chi_Minh` defensively in runtime helpers.
- **Redis counter lost or expired early** -> Rebuild from `token_usage_logs` for the current calendar week.
- **Race around weekly reset boundary** -> Compute start/end/TTL from a single current timestamp per operation and use `< next_monday_start_utc` for DB rebuild.
- **Small overspend under concurrent requests** -> Existing pre-request checks are approximate because final costs are known after upstream responses; weekly enforcement should match current budget-counter semantics.
- **Timezone changes mid-week** -> New checks use the current configured timezone, which may change the computed weekly key and reporting window. This is acceptable for an admin-controlled global setting.
- **New dependency footprint** -> Timezone handling may require a crate such as `chrono-tz`; keep usage isolated in subscription/settings helpers.

## Migration Plan

1. Add migration 045 with default timezone and nullable weekly cost limit columns.
2. Update backend models, SQL queries, and admin settings/plan/subscription assignment routes.
3. Add Redis cache helpers for timezone and use them in weekly boundary/counter helpers.
4. Add enforcement and counter updates in subscription logic.
5. Extend public usage API and frontend pages.
6. Run `just check` and fix Rust/frontend issues.

Rollback is straightforward for code but database rollback would require removing the added columns and any Redis weekly counter keys can expire naturally.

## Open Questions

- Whether plan updates should auto-sync `weekly_cost_limit_usd` to active assigned subscriptions, similar to TPM/RPM behavior, or remain snapshot-only. The implementation should follow existing plan-field conventions if a sync pattern already exists.
