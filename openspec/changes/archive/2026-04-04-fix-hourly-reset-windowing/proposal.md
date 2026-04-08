## Why

The `hourly_reset` subscription windowing uses continuous rolling intervals anchored to `activated_at`, so windows advance even when no requests are made. A user returning after an idle period gets a partially-elapsed window instead of a fresh full window, causing unexpected budget exhaustion and a confusing experience.

## What Changes

- Replace `compute_window_idx` (rolling interval from activation time) with demand-driven window tracking in Redis
- New Redis key `sub_window_start:{sub_id}` stores the epoch timestamp when the current window started, with TTL equal to the window duration so it auto-expires when the window ends
- `get_total_cost` and `get_model_cost` read the window start from Redis instead of computing it from `activated_at`; if no key exists (no active window), they return 0.0
- `update_cost_counters` uses SETNX semantics to create a window start on the first request, then scopes cost counter keys to that window start epoch
- `rebuild_total_cost` and `rebuild_model_cost` accept a window start timestamp and query only usage within that window's time range
- `compute_window_reset_at` in `public/usage.rs` becomes async and reads the Redis key to determine when the current window expires; returns `None` when no window is active
- Remove `compute_window_idx` — no longer needed

## Capabilities

### New Capabilities

- `demand-driven-window-tracking`: Redis-based window lifecycle — windows start on first request, expire automatically via TTL, and do not advance during idle periods

### Modified Capabilities

- `subscription-cost-tracking`: Cost read and write paths now scope to a demand-driven window start epoch rather than a rolling index computed from activation time
- `public-usage-api`: `reset_at` field becomes optional (`Option<DateTime<Utc>>`); returns `null` when no active window exists

## Impact

- `viber-router-api/src/subscription.rs` — core windowing logic rewritten
- `viber-router-api/src/routes/public/usage.rs` — `compute_window_reset_at` becomes async, return type changes
- `viber-router-api/src/routes/proxy.rs` — `update_cost_counters` call sites may need minor adjustments if signature changes
- No database migration required; `activated_at` column is retained for package expiration tracking
- Redis key namespace gains `sub_window_start:{sub_id}`
