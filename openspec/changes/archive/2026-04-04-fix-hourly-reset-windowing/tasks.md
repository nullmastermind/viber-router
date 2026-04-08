## 1. Redis Window Helpers (subscription.rs)

- [x] 1.1 Add `get_window_start(state: &AppState, sub_id: i32) -> Option<i64>`: GET `sub_window_start:{sub_id}` from Redis, parse as i64, return None on miss or error
- [x] 1.2 Add `ensure_window_start(state: &AppState, sub_id: i32, reset_hours: i32) -> Option<i64>`: SET `sub_window_start:{sub_id}` NX EX `reset_hours * 3600` to current epoch, then GET to return the stored value; return None and log warning on Redis error ← (verify: SETNX semantics work correctly — concurrent calls return the same epoch; TTL is set to reset_hours * 3600)

## 2. Update Cost Read Functions (subscription.rs)

- [x] 2.1 Rewrite `get_total_cost`: replace `compute_window_idx` call with `get_window_start`; if None return 0.0; if Some(ws) use key `sub_cost:{sub_id}:ws:{ws}`; call `rebuild_total_cost(state, sub, ws)` on Redis miss
- [x] 2.2 Rewrite `get_model_cost`: same pattern as 2.1 using key `sub_cost:{sub_id}:ws:{ws}:m:{model}` and `rebuild_model_cost(state, sub, model, ws)` on miss ← (verify: returning 0.0 when no window is active; correct key used when window exists; rebuild triggered on miss)

## 3. Update Cost Rebuild Functions (subscription.rs)

- [x] 3.1 Update `rebuild_total_cost` signature to accept `window_start: i64` instead of computing from `activated_at`; update SQL query to `WHERE subscription_id = $1 AND created_at >= $2 AND created_at < $3` with `$2 = DateTime::from_timestamp(window_start)` and `$3 = $2 + reset_hours * 3600 seconds`
- [x] 3.2 Update `rebuild_model_cost` with the same signature and query changes as 3.1 ← (verify: SQL time bounds match the window exactly; rebuild result is SET in Redis with correct TTL)

## 4. Update Cost Write Function (subscription.rs)

- [x] 4.1 Update `update_cost_counters`: for subscriptions with `reset_hours` set, call `ensure_window_start` to get `ws`; if None skip counter update and log warning; if Some(ws) use keys `sub_cost:{sub_id}:ws:{ws}` and `sub_cost:{sub_id}:ws:{ws}:m:{model}` with TTL `reset_hours * 3600` ← (verify: new window is created on first request; subsequent requests in same window increment the same key; TTL is applied to counter keys)

## 5. Remove compute_window_idx (subscription.rs)

- [x] 5.1 Delete the `compute_window_idx` function (no longer referenced after steps 2–4)

## 6. Update compute_window_reset_at (public/usage.rs)

- [x] 6.1 Make `compute_window_reset_at` async and add `state: &AppState` parameter
- [x] 6.2 Change return type from `DateTime<Utc>` to `Option<DateTime<Utc>>`
- [x] 6.3 Implement: GET `sub_window_start:{sub_id}` from Redis; if None return None; if Some(ws) return `Some(DateTime::from_timestamp(ws) + reset_hours * 3600 seconds)`
- [x] 6.4 Update all call sites of `compute_window_reset_at` in `usage.rs` to await and handle `Option` ← (verify: returns None when no window active; returns correct reset time when window exists; public usage API response has window_reset_at: null when no window)

## 7. Adjust proxy.rs Call Sites (if needed)

- [x] 7.1 Review `update_cost_counters` call sites at proxy.rs lines ~1275 and ~1733; update argument list if the function signature changed ← (verify: proxy.rs compiles without errors; cost counters are updated on proxied requests)

## 8. Final Check

- [x] 8.1 Run `just check` and fix all lint and type errors ← (verify: `just check` exits 0 with no warnings or errors)
