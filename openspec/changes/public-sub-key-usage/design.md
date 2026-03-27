## Context

The Viber Router admin UI provides sub-key usage and subscription data through authenticated admin endpoints. Sub-key holders (end users) have no self-service way to check their usage. The backend already tracks all necessary data: `token_usage_logs` (per-request with `group_key_id`), `key_subscriptions` (with cost tracking via Redis), and `group_keys` (with group association).

Current architecture:
- Admin endpoints under `/api/admin/*` protected by `admin_auth` middleware
- Proxy endpoints under `/v1/*` authenticated by group/sub-key
- Health endpoint at `/health` — fully public
- Frontend uses hash-mode routing with a `beforeEach` guard that redirects non-`/login` paths to `/login` when no admin token exists
- Rate limiting exists for group-server pairs using Redis INCR+EXPIRE pattern

## Goals / Non-Goals

**Goals:**
- Public endpoint that returns usage + subscription data for a valid sub-key
- IP-based rate limiting to prevent key enumeration/brute-force
- Frontend page accessible without admin login
- Hide upstream server information from public responses

**Non-Goals:**
- User accounts or authentication system for sub-key holders
- Ability to modify subscriptions or key settings from the public page
- Historical usage beyond 30 days
- Real-time usage updates (WebSocket/SSE)
- Custom date range selection on the public page

## Decisions

**1. Single endpoint returning all data**
Return key info, usage, and subscriptions in one response rather than separate endpoints. Rationale: minimizes round-trips for a read-only page; the data set is small enough to combine.

**2. Usage query grouped by model only**
The existing admin token_usage query groups by `(server_id, server_name, model)`. The public endpoint uses a simpler query grouping by `model` only, with `SUM(cost_usd)` from the pre-calculated column. This hides upstream server identity completely — no server_id, no server_name, no JOIN to servers table.

**3. IP-based rate limiting via Redis**
Reuse the existing INCR+EXPIRE pattern from `rate_limiter.rs`. Key format: `rl:pub:<ip>`. 30 requests per 60-second window. Fail-open on Redis errors (consistent with existing rate limiter behavior). Extract client IP from `x-forwarded-for` header (reverse proxy) or connection info.

**4. Generic error for invalid keys**
Return the same 403 response for non-existent keys and deactivated keys. Prevents attackers from distinguishing valid-but-inactive keys from non-existent ones.

**5. Subscription window_reset_at computation**
For `hourly_reset` subscriptions: compute `window_start = activated_at + (window_idx * reset_hours * 3600)`, then `window_reset_at = window_start + reset_hours * 3600`. Return as ISO 8601 timestamp. For `fixed` subscriptions: `window_reset_at` is null (no reset concept).

**6. Frontend route outside MainLayout**
Follow the `/login` pattern: top-level route without layout wrapper. Update the `beforeEach` guard to exempt paths starting with `/usage`.

## Risks / Trade-offs

**[Key enumeration via timing]** → The endpoint validates the key against the database, which could leak existence via response time differences. Mitigation: the generic error message is the primary defense; timing attacks require many requests which the rate limiter throttles.

**[Rate limit bypass via IP spoofing]** → If behind a reverse proxy, `x-forwarded-for` can be spoofed. Mitigation: acceptable for an internal tool; the rate limit is a speed bump, not a security boundary.

**[Stale cost_used from Redis]** → Redis cost counters may lag slightly behind actual usage. Mitigation: acceptable for a read-only dashboard; the existing admin UI has the same behavior.
