## Context

Viber Router routes requests from sub-keys to upstream AI servers via a failover waterfall. Sub-keys already support subscriptions (fixed, hourly_reset, pay_per_request) to budget-control usage. The proxy's `check_subscriptions()` function returns a `SubCheckResult` (Allowed, Blocked, etc.) that the proxy handler uses to select a charging subscription before entering the group server waterfall.

The bonus sub-type introduces a fundamentally different concept: rather than budget-gating the group's servers, a bonus subscription carries its own upstream server that the proxy should try first. This is a routing concern (which server to call) layered on top of the existing budget concern (which subscription to charge).

## Goals / Non-Goals

**Goals:**
- Allow admins to bind one or more dedicated upstream servers to a sub-key, each tried before the group's normal servers.
- Fall back gracefully to group servers if all bonus servers return errors.
- Let the public usage page display per-model request counts and optional quota data for bonus subscriptions.
- Keep the existing non-bonus subscription logic completely unchanged.

**Non-Goals:**
- No cost tracking for bonus requests (cost_usd remains NULL).
- No caching or blacklisting of failed bonus servers between requests.
- No OpenAI endpoint support (only `/v1/messages`).
- No plan-based creation for bonus subscriptions — direct creation only.
- No parsing of provider-specific response headers (e.g., x-ratelimit-*).
- No quota polling on a schedule; quota is fetched per public usage page load.

## Decisions

### 1. New SubCheckResult variant rather than a separate code path

**Decision**: Add `BonusServers { servers: Vec<BonusServer>, fallback_subscription: Option<(Uuid, Option<f64>)> }` to `SubCheckResult` instead of handling bonus logic inside or after the existing check.

**Rationale**: `check_subscriptions()` already encapsulates subscription selection logic. Keeping bonus resolution there preserves the single responsibility boundary and ensures bonus subs are evaluated with full knowledge of non-bonus subs (needed to compute `fallback_subscription`).

**Alternative considered**: A second function call (`check_bonus_subscriptions()`) in the proxy handler. Rejected because it would duplicate the non-bonus resolution path and require the proxy to call both functions in sequence.

### 2. Bonus servers tried in FIFO order (created_at ASC), independent of non-bonus priority sort

**Decision**: Bonus subs are collected by `created_at ASC`. Non-bonus subs continue to use the existing priority sort (hourly_reset > pay_per_request > fixed, then FIFO within type).

**Rationale**: Bonus subs model dedicated seats, not interchangeable budget pools. FIFO reflects intended preference order set by the admin (first bonus added = primary seat). Mixing bonus into the existing type_order sort would break that intent.

### 3. Bonus creation without plan_id, with hardcoded defaults

**Decision**: The POST endpoint checks: if `bonus_name`, `bonus_base_url`, and `bonus_api_key` are all present in the body (and no `plan_id`), treat as a bonus creation. Hardcode `sub_type = 'bonus'`, `cost_limit_usd = 0`, `model_limits = {}`, `model_request_costs = {}`, `duration_days = 36500`, `reset_hours = NULL`, `rpm_limit = NULL`, `plan_id = NULL`.

**Rationale**: Bonus subs have no cost semantics. Hardcoding the irrelevant fields avoids exposing meaningless form fields to the admin and prevents accidental misconfiguration.

### 4. Quota URL fetched on each public usage page load, no caching

**Decision**: If `bonus_quota_url` is set, the backend makes a GET request with a 5-second timeout on every public usage API call. Errors result in an empty quotas array (fail silent).

**Rationale**: Quota data is time-sensitive (it reflects current usage at the external provider). Caching would require invalidation logic. Given that the public usage page is a low-frequency human-facing page, per-request fetching is acceptable. The 5-second timeout prevents the public usage endpoint from hanging if the quota server is slow.

**Alternative**: Server-side background polling, storing quota in the database. Rejected as over-engineered for v1.

### 5. Usage logging: subscription_id = bonus sub id, cost_usd = NULL

**Decision**: When a bonus server succeeds, log the token usage with `subscription_id` set to the bonus subscription's UUID and `cost_usd = NULL`.

**Rationale**: Preserves attribution (which bonus sub handled the request) while being explicit that no cost was incurred. This is the source of `bonus_usage` per-model request counts in the public usage API.

### 6. Bonus-only sub-keys can still use group servers as fallback

**Decision**: If `BonusServers.fallback_subscription` is `None` (sub-key has no non-bonus active subscriptions) and all bonus servers fail, the proxy proceeds to the group server waterfall without subscription tracking.

**Rationale**: Bonus is additive — it should never be strictly more restrictive than having no subscription. A sub-key with only bonus subs that exhausts all bonus servers should not be penalized; it still gets group server access.

## Risks / Trade-offs

- **Outbound HTTP in proxy hot path (quota URL)**: The quota fetch is only on the public usage page, not the proxy path. No hot-path risk.
- **Bonus server availability**: If a bonus server consistently returns errors, every request for that sub-key will incur one failed HTTP round-trip per bonus server before reaching group servers. Mitigation: the admin should remove or disable bonus subs for unreliable servers; no automatic caching of failures in v1.
- **Database migration** → Requires an ALTER TABLE on `key_subscriptions`. This is a non-destructive additive migration (new nullable columns + updated CHECK constraint). The plan: deploy migration first, then deploy new backend binary.
- **`SubCheckResult` exhaustiveness** → Adding a new enum variant requires all existing match arms to be updated. Rust's exhaustive match ensures this is a compile-time error, not a runtime risk.
- **Admin-supplied quota URL**: The backend makes an outbound GET to an admin-controlled URL. This is an internal tool with trusted admins (per CLAUDE.md security notes), so no SSRF restrictions are needed.

## Migration Plan

1. Run migration `035_add_bonus_subscription_columns.sql` which:
   - Adds 5 nullable columns to `key_subscriptions`.
   - Drops and recreates the `sub_type` CHECK constraint on `key_subscriptions` to include `'bonus'`.
   - Drops and recreates the `sub_type` CHECK constraint on `subscription_plans` to include `'bonus'`.
2. Deploy the new backend binary (includes model changes, new SubCheckResult variant, proxy bonus waterfall, public usage API changes).
3. Deploy the new frontend build (GroupDetailPage, PublicUsagePage, useSubscriptionType changes).
4. No rollback data concerns: bonus columns are nullable with no defaults; removing a bonus sub is a soft cancel via existing PATCH endpoint.
