## Context

The viber-router backend tracks subscription cost usage in Redis. For subscriptions with `reset_hours` set, the current implementation computes a `window_idx` by dividing elapsed time since `activated_at` by the window duration. This creates a continuous rolling schedule: window 0 is `[activated_at, activated_at + reset_hours)`, window 1 is `[activated_at + reset_hours, activated_at + 2*reset_hours)`, and so on. Windows advance on a fixed clock regardless of whether any requests were made.

The result is that a user who is idle for longer than one window period returns to a partially-elapsed window, not a fresh one. The budget appears partially consumed even though no requests were made during the gap.

All cost tracking is in Redis. The database (`token_usage_logs`) is the source of truth for rebuilds. No schema changes are needed.

## Goals / Non-Goals

**Goals:**
- Windows start only when a request arrives (demand-driven)
- A window lasts exactly `reset_hours` from the first request in that window
- When a window expires with no new request, no new window is created — the budget is effectively frozen at 0 until the next request
- The public usage API reflects the active window's reset time, or `null` when no window is active
- Fix applies to all subscriptions that use `reset_hours` (both `hourly_reset` and `pay_per_request` with reset)

**Non-Goals:**
- Changing the database schema or adding new tables
- Altering how `activated_at` is used for package expiration
- Changing behavior for fixed subscriptions (no `reset_hours`)
- Retroactively migrating existing window state in Redis

## Decisions

### Decision 1: Redis TTL as the window timer

Use a Redis key `sub_window_start:{sub_id}` with TTL = `reset_hours * 3600` seconds. The key stores the epoch timestamp (i64) of when the window started. When the TTL expires, the key disappears — no window is active. The next request creates a new key via SETNX.

Alternatives considered:
- Store window end time instead of start time: rejected — start time is more useful for rebuilding cost from DB (need `WHERE created_at >= start AND created_at < start + duration`)
- Use a separate expiry key: rejected — unnecessary complexity; TTL on the start key is sufficient
- Keep `window_idx` but reset it on activity: rejected — requires a separate "last active" key and more complex logic; TTL-based approach is simpler and self-cleaning

### Decision 2: SETNX semantics for window creation

`ensure_window_start` uses SET with NX (set if not exists) and EX (TTL). If the key already exists, the SET is a no-op and the existing value is returned via a subsequent GET. This handles concurrent requests correctly: only the first request of a window sets the start time; all others read the same value.

Alternatives considered:
- Lua script for atomic SETNX+GET: more correct but adds complexity; the two-command approach (SET NX EX, then GET) is acceptable because a race between SET and GET is benign — the worst case is two requests both think they are "first" and both set the same approximate timestamp, but SETNX ensures only one wins
- Use Redis transactions (MULTI/EXEC): unnecessary overhead for this use case

### Decision 3: Cost counter key includes window start epoch

New key pattern: `sub_cost:{sub_id}:ws:{window_start_epoch}` and `sub_cost:{sub_id}:ws:{window_start_epoch}:m:{model}`.

The epoch is embedded in the key so that counters from different windows never collide. Old window keys become orphaned when the window expires (the `sub_window_start` key disappears), but the cost counter keys themselves do not need a TTL — they will be naturally unreachable once the window start key is gone. A background cleanup is not required for correctness.

Alternatives considered:
- Reuse existing `w:{window_idx}` key pattern: rejected — `window_idx` is no longer computed; using epoch is more explicit and avoids confusion with the old scheme
- Set TTL on cost counter keys: could be added as a cleanup measure but is not required for correctness; omitted to keep the implementation simple

### Decision 4: `get_window_start` returns `None` on Redis miss → cost = 0

If `sub_window_start:{sub_id}` does not exist in Redis (either no window is active, or Redis is temporarily unavailable), `get_total_cost` and `get_model_cost` return `0.0`. This means the subscription appears to have no cost used, which allows the request to proceed. The cost is then recorded correctly by `update_cost_counters` which calls `ensure_window_start`.

Trade-off: if Redis is unavailable, cost checks are bypassed (requests are allowed). This matches the existing behavior for Redis failures and is acceptable for an internal admin tool.

### Decision 5: `compute_window_reset_at` becomes async

The function needs to read from Redis to determine when the current window expires. Making it async is the straightforward approach in an async Axum handler. The function signature gains `&AppState`.

Return type changes from `DateTime<Utc>` to `Option<DateTime<Utc>>` — `None` when no active window exists.

## Risks / Trade-offs

- [Orphaned cost counter keys] Keys like `sub_cost:{sub_id}:ws:{old_epoch}:m:{model}` accumulate in Redis over time with no TTL. Mitigation: these keys are small and bounded by the number of windows a subscription has ever had. A future cleanup job can scan and remove keys whose epoch is older than the subscription's `reset_hours`. Not required for correctness.

- [Redis unavailable during ensure_window_start] If Redis is down when a request is processed, `ensure_window_start` returns `None`, cost counters are not updated, and the request is allowed. Mitigation: same as current behavior; log a warning. Acceptable for internal tooling.

- [Race condition on window boundary] If a window expires between `get_total_cost` (returns 0, window just expired) and `update_cost_counters` (creates new window), the cost goes to the new window. This is correct behavior — the new window starts fresh.

- [Existing Redis keys from old scheme] After deployment, old `sub_cost:{sub_id}:w:{window_idx}` keys remain in Redis until their TTL expires. They are unreachable by the new code and will expire naturally. No data loss.

## Migration Plan

1. Deploy the updated backend binary. No database migration needed.
2. Old `w:{window_idx}` cost counter keys expire naturally via their existing TTLs.
3. On the first request after deployment for each subscription, `ensure_window_start` creates a new `sub_window_start:{sub_id}` key and the new key scheme takes effect.
4. Rollback: revert to previous binary. Old `w:{window_idx}` keys may have expired, so cost counters will rebuild from DB on next access (existing rebuild logic handles this).

## Open Questions

- Should orphaned `sub_cost:{sub_id}:ws:{epoch}` keys be cleaned up proactively? Current decision: no, defer to a future chore if Redis memory becomes a concern.
- Should `window_reset_at: null` in the public usage API response be documented as a breaking change for API consumers? Current decision: treat as a bug fix — the field was previously always populated for `hourly_reset` subscriptions, but returning `null` when no window is active is more correct.
