## Context

Viber Router is a proxy that routes API requests to upstream LLM providers. Groups define routing rules with a priority-ordered chain of servers. Each group-server assignment has per-assignment configuration: priority, model mappings, enabled toggle, circuit breaker settings, and cost rate multipliers.

Currently, the proxy iterates through servers in priority order, skipping servers that lack an API key or have an open circuit breaker. There is no mechanism to limit how many requests a server receives within a time window. Lower-capacity servers in a group can become overloaded when traffic spikes.

The circuit breaker module (`circuit_breaker.rs`) provides the closest existing pattern: Redis-based counters with TTL windows, checked in the proxy failover loop, with fail-open semantics on Redis errors.

## Goals / Non-Goals

**Goals:**
- Allow admins to configure per group-server rate limits (max requests per time window)
- Proxy skips rate-limited servers and tries the next server in the chain
- Return a distinct 429 error when all servers are rate-limited
- Mirror the circuit breaker pattern for consistency (Redis counters, all-or-nothing validation, fail-open)

**Non-Goals:**
- Global per-server rate limiting (across all groups) — out of scope
- Concurrent request limiting (semaphore-style) — different feature
- Rate limiting by token count or cost — out of scope
- Rate limit analytics/dashboard — out of scope

## Decisions

### 1. Storage: two nullable INTEGER columns on `group_servers`

Add `max_requests INTEGER` and `rate_window_seconds INTEGER` to the `group_servers` table. Both nullable, defaulting to NULL (no rate limit). Migration: `024_add_rate_limit_to_group_servers.sql`.

**Why over a separate table**: Rate limit config is per group-server assignment, same as circuit breaker. Keeping it on the same row avoids JOINs and matches the existing pattern.

### 2. Counter mechanism: Redis INCR + TTL

Redis key: `rl:{group_id}:{server_id}`. On each request:
1. GET current count
2. If count >= max_requests → skip server
3. If count < max_requests → INCR, set TTL to `rate_window_seconds` if key is new (count == 1 after INCR)

This mirrors `circuit_breaker::record_error` which uses the same INCR + conditional EXPIRE pattern.

**Why not Lua script**: The INCR+EXPIRE pattern is already proven in the circuit breaker. Two Redis commands per check is acceptable for this use case.

### 3. Counting strategy: optimistic increment (before send)

INCR the counter before sending the upstream request. If the request fails and the proxy fails over to the next server, the counter still reflects the attempt. This is acceptable because the server did receive (or was about to receive) the request.

**Why not after-send**: Would require tracking success/failure and decrementing on failure, adding complexity. The server still processes the request even if it returns an error.

### 4. Module: dedicated `rate_limiter.rs`

Create `viber-router-api/src/rate_limiter.rs` with two public functions:
- `is_rate_limited(redis, group_id, server_id, max_requests, window_seconds) -> bool` — check if limit reached
- `increment_rate_limit(redis, group_id, server_id, window_seconds)` — INCR counter

Mirrors `circuit_breaker.rs` structure. Imported in `proxy.rs` as `crate::rate_limiter`.

### 5. Proxy integration: check before `any_server_attempted`, after circuit breaker

In the failover waterfall loop, the rate limit check goes after the circuit breaker check and before `any_server_attempted = true`. This ensures rate-limited servers are treated like circuit-broken servers — skipped without counting as "attempted". A `any_rate_limited` flag tracks whether any server was skipped due to rate limiting, to distinguish the 429 message.

### 6. Validation: all-or-nothing, values >= 1

Both `max_requests` and `rate_window_seconds` must be set together or both NULL. Values must be >= 1. Follows the exact same pattern as `validate_cb_fields`. A new `validate_rate_limit_fields` function in `group_servers.rs`.

### 7. UI: Edit Server dialog section + badge

Rate limit config in the Edit Server dialog, below the Circuit Breaker section. Two number inputs: "Max Requests" and "Window (seconds)". Badge on server list showing `{max_requests}/{rate_window_seconds}s` format (e.g., "100/60s") when rate limit is configured.

### 8. Redis failure: fail open

If Redis is unavailable during rate limit check, the request proceeds (server is not skipped). Consistent with circuit breaker behavior.

## Risks / Trade-offs

- **Counter not perfectly accurate under high concurrency**: Between GET (check) and INCR (increment), another request could also pass the check. This is acceptable — the window is very small and the limit is a soft cap, not a hard security boundary. → Mitigation: Use INCR first, then check count (atomic increment), same as CB pattern.
- **Optimistic counting inflates counter on failures**: If a request fails and fails over, the counter for the failed server is still incremented. → Mitigation: Acceptable trade-off. The server did receive the attempt. Admins can set slightly higher limits to account for this.
- **No per-model or per-key rate limiting**: This is per group-server only. → Mitigation: Explicitly out of scope. Can be added later as a separate feature.
