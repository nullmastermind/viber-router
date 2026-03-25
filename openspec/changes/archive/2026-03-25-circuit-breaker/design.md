## Context

The proxy engine implements a failover chain: when a request to server N fails with a configured status code or connection error, it tries server N+1. Currently there is no mechanism to remember that a server has been failing — every new request retries the same broken server, wasting time and adding load. The `is_enabled` toggle exists for manual disable but requires admin intervention.

The system already uses Redis for caching (`GroupConfig`) and Telegram for alerting on upstream errors.

## Goals / Non-Goals

**Goals:**
- Auto-disable a server within a group after it exceeds a configurable error threshold within a time window
- Auto re-enable after a configurable cooldown period (no admin action needed)
- Notify via Telegram on both state transitions (trip and re-enable)
- Allow per group-server configuration (same server can have different thresholds in different groups)
- Zero-overhead when circuit breaker is not configured (null fields = disabled)

**Non-Goals:**
- Half-open state (gradual traffic ramp-up) — out of scope, full re-enable after cooldown
- Persisting circuit breaker state to DB — Redis-only, lost on restart is acceptable
- Global/server-level circuit breaker — only per group-server assignment
- Blocking requests when all servers are circuit-broken — returns 429 same as current behavior

## Decisions

### 1. State storage: Redis with TTL keys

**Choice**: Two Redis key patterns per (group_id, server_id):
- `cb:err:{group_id}:{server_id}` — error counter, TTL = `cb_window_seconds`. INCR on each error, auto-expires.
- `cb:open:{group_id}:{server_id}` — circuit open marker, TTL = `cb_cooldown_seconds`. EXISTS = skip server.

**Why over in-memory**: Survives process restarts within TTL. Shared across multiple API instances if scaled horizontally. Redis is already a dependency.

**Why over DB**: No write amplification on every error. Counter operations are O(1) in Redis. Circuit state is ephemeral by nature.

### 2. Error definition: failover codes + connection errors

**Choice**: Any event that triggers `continue` in the failover loop counts as an error:
- Status code in `failover_status_codes` (group-configured)
- Connection error (status = 0)
- TTFT timeout (status set to 0)

**Why**: These are exactly the conditions where the proxy already considers the server "failed" for this request.

### 3. Circuit breaker check placement: inside failover loop, before request

**Choice**: In `proxy.rs`, after checking `is_enabled` and before sending the upstream request:
```
if cb configured && cb:open key exists → skip server (continue)
```

On error, after recording failover attempt:
```
if cb configured → INCR cb:err key (set TTL if new) → if count >= max → SET cb:open key → alert
```

### 4. Re-enable detection: check-on-next-request

**Choice**: When proxy checks `cb:open` and the key doesn't exist (expired), treat as re-enabled. Send Telegram alert at this point using a one-time `cb:realerted:{group_id}:{server_id}` key to avoid duplicate alerts.

**Why over keyspace notifications**: Simpler, no background subscriber needed. Alert delay is negligible with any traffic.

### 5. Configuration validation: all-or-nothing

**Choice**: All three fields (`cb_max_failures`, `cb_window_seconds`, `cb_cooldown_seconds`) must be either all null or all non-null. Validated in the `update_assignment` handler. Minimum values: max_failures >= 1, window_seconds >= 1, cooldown_seconds >= 1.

### 6. Circuit status API for frontend

**Choice**: `GET /api/admin/groups/{group_id}/circuit-status` returns circuit state for all servers in the group by checking `cb:open` keys and their remaining TTL. Frontend polls every 10s when any circuit is open.

### 7. GroupConfig includes circuit breaker fields

The `GroupConfig` struct (cached in Redis for proxy use) will include `cb_max_failures`, `cb_window_seconds`, `cb_cooldown_seconds` per server. This means the proxy reads CB config from the cached config, not from DB on every request.

## Risks / Trade-offs

- **Redis restart loses circuit state** → Acceptable. Circuits reset to closed, worst case is a few extra errors before re-tripping. No data loss.
- **Race condition on INCR** → Multiple concurrent requests could increment past threshold simultaneously. Mitigated: SET cb:open is idempotent, multiple SETs just refresh TTL. Telegram alert uses NX key to send only once.
- **Cache staleness** → CB config is part of GroupConfig cached in Redis. Config changes invalidate cache (existing pattern). New requests pick up updated config.
- **All servers circuit-broken** → Returns 429 to client. Same as current "all servers exhausted" behavior. No special handling needed.
