## Context

The proxy already enforces subscription budgets and RPM limits for billing endpoints. RPM uses Redis fixed-window counters and can skip a candidate subscription when a request-rate limit is reached. TPM needs to use the same subscription identity and Redis-backed fixed-window style, but its behavior is deliberately different: the chosen eligible non-bonus subscription waits for token capacity instead of falling through to another subscription.

The system also snapshots plan configuration into `key_subscriptions`, exposes subscription data in admin and public APIs, and displays plan/subscription fields in the Vue/Quasar admin UI. TPM therefore crosses database schema, models, plan sync, assignment flows, proxy enforcement, token accounting, and UI surfaces.

## Goals / Non-Goals

**Goals:**
- Add nullable TPM limits to subscription plans and subscription instances without changing behavior for existing records.
- Enforce TPM on active non-bonus subscriptions for `/v1/messages` and `/v1/chat/completions` billing requests.
- Wait asynchronously for Redis fixed-window reset when a selected subscription has reached its TPM limit, retrying up to five windows before returning 429.
- Increment TPM counters using actual input plus output tokens after upstream responses complete.
- Keep Redis failures fail-open for both checks and increments.
- Keep plan-to-subscription snapshot and sync behavior consistent with existing RPM patterns.
- Expose TPM configuration in admin APIs, public usage API, and admin UI.

**Non-Goals:**
- No per-model TPM limits.
- No sliding-window, leaky-bucket, or token-bucket algorithm.
- No TPM enforcement for bonus subscriptions.
- No TPM analytics, dashboard, historical reporting, or configurable retry timeout.
- No pre-request token estimation.

## Decisions

### Store TPM as nullable `FLOAT8` on plans and subscriptions

Use `tpm_limit FLOAT8 NULL` on both `subscription_plans` and `key_subscriptions`, matching the optional nature and numeric shape of existing limit fields. `NULL` means unlimited/no TPM enforcement. Subscription assignment snapshots the current plan value so each subscription remains stable unless explicitly synced.

Alternative considered: store TPM only on plans and join at runtime. This was rejected because subscriptions already snapshot plan fields and can diverge from plans after assignment.

### Use Redis fixed-window counters named `sub_tpm:{subscription_id}`

TPM uses one Redis integer counter per subscription with a 60-second TTL. After a response completes, the implementation increments by `input_tokens + output_tokens`; if the new total equals the added amount, the key was newly created and gets `EXPIRE 60`. Existing counters do not have their TTL extended.

Alternative considered: always set `EXPIRE 60` after increment. This was rejected because it would extend the window on every request and become a sliding inactivity timeout rather than a fixed window.

### Enforce TPM after subscription selection, not during subscription selection

`check_subscriptions()` returns the selected `Allowed` subscription with its `tpm_limit`. The proxy then runs TPM waiting for that selected subscription. If the TPM window is full, the request waits for reset rather than skipping to another subscription. This preserves the explicit behavioral difference from RPM.

Alternative considered: treat TPM like RPM inside the selection loop and skip limited subscriptions. This was rejected because the requested behavior is to wait on the chosen subscription instead of falling through.

### Count actual tokens after response completion

TPM increments use actual token usage parsed from completed upstream responses. This includes input and output tokens. For streaming responses, increment after the SSE stream completes and token usage has been parsed. Requests with unavailable token counts do not add estimated usage.

Alternative considered: estimate input tokens before forwarding and reserve capacity. This was rejected because the requirement is post-response actual usage and because output tokens are unknown before completion.

### Fail open on Redis errors

If Redis is unavailable during TPM check, TTL lookup, sleep decision, or increment, the proxy allows the request to continue and logs the issue. This matches the project’s existing rate-limit fail-open posture and avoids outages caused by cache failures.

Alternative considered: fail closed when Redis cannot verify token capacity. This was rejected to preserve availability for an internal routing tool.

### Sync TPM through explicit endpoint and plan update auto-sync

Plan updates propagate changed TPM limits to active subscriptions, mirroring RPM auto-sync. Admins also get `POST /api/admin/subscription-plans/{id}/sync-tpm` to force active subscription snapshots to match a plan. Cache invalidation follows the existing plan sync/RPM invalidation pattern.

Alternative considered: no sync endpoint. This was rejected because admins already have plan sync workflows and need parity with RPM behavior.

## Risks / Trade-offs

- Waiting proxy requests can occupy tasks for up to roughly five minutes → Mitigate by using `tokio::time::sleep` so waits are non-blocking and by limiting retries to five.
- Post-response accounting can allow a burst to exceed the configured TPM during a fresh or under-limit window → This is accepted because pre-request estimation is explicitly out of scope.
- Redis fixed windows can create boundary bursts → This matches the existing RPM pattern and keeps implementation simple.
- Missing token usage from unusual upstream responses can undercount TPM → Mitigate by incrementing only when parsed token counts are available and preserving existing token extraction behavior.
- `FLOAT8` limits with Redis integer counters require comparison discipline → Treat token counts as integers converted for comparison against the floating-point limit, and document that fractional TPM values are not practically useful.
- Public API response shape gains a nullable field → This should be backward compatible for clients that ignore unknown fields.

## Migration Plan

1. Add migration `044_add_tpm_limit.sql` with nullable `tpm_limit FLOAT8` columns on `subscription_plans` and `key_subscriptions`.
2. Deploy backend changes that read/write nullable TPM fields and tolerate null values.
3. Deploy frontend changes for TPM display/edit/sync.
4. Existing records remain unlimited until admins set TPM values on plans or subscriptions are synced.

Rollback is safe at application level by deploying previous code while leaving nullable columns unused. A database rollback would drop the two nullable columns if required by the deployment process.

## Open Questions

- None. TPM retry count, Redis key format, fail-open behavior, scope, and token accounting method are fixed by the plan.
