## 1. Backend: Public Route Module

- [x] 1.1 Create `viber-router-api/src/routes/public/mod.rs` with public router function
- [x] 1.2 Create `viber-router-api/src/routes/public/usage.rs` with usage handler
- [x] 1.3 Register `mod public` in `viber-router-api/src/routes/mod.rs` and nest `/api/public` in the top-level router ← (verify: route is accessible without admin auth, no middleware applied)

## 2. Backend: IP Rate Limiting

- [x] 2.1 Add `is_ip_rate_limited` and `increment_ip_rate_limit` functions to `viber-router-api/src/rate_limiter.rs` using Redis INCR+EXPIRE pattern with key format `rl:pub:<ip>`, 30 req/60s window, fail-open on Redis errors
- [x] 2.2 Integrate rate limit check in the usage handler — extract client IP from `x-forwarded-for` or connection info, return 429 + `Retry-After: 60` header when limited ← (verify: rate limit returns 429 with Retry-After header, fails open on Redis error)

## 3. Backend: Usage Handler Logic

- [x] 3.1 Parse `key` query parameter, return 400 if missing
- [x] 3.2 Lookup sub-key in `group_keys` table joined with `groups` to get key_name and group_name; return generic 403 `"Invalid or inactive key"` for non-existent or `is_active = false` keys
- [x] 3.3 Query `token_usage_logs` aggregated by model only (no server info) for last 30 days where `group_key_id` matches — fields: model, total_input_tokens, total_output_tokens, total_cache_creation_tokens, total_cache_read_tokens, request_count, cost_usd
- [x] 3.4 Query all `key_subscriptions` for the sub-key (all statuses), enrich each with `cost_used` via `crate::subscription::get_total_cost`, compute `window_reset_at` for `hourly_reset` type using `activated_at + (window_idx + 1) * reset_hours * 3600`
- [x] 3.5 Return JSON response with `key_name`, `group_name`, `usage[]`, `subscriptions[]` ← (verify: response contains no server_id/server_name, subscriptions include cost_used and window_reset_at, all subscription statuses returned)

## 4. Frontend: Routing

- [x] 4.1 Add `/usage` and `/usage/:key` routes to `src/router/routes.ts` as top-level routes (outside MainLayout, same pattern as `/login`)
- [x] 4.2 Update `beforeEach` guard in `src/router/index.ts` to exempt paths starting with `/usage` from admin token check ← (verify: unauthenticated user can access /usage without redirect to /login)

## 5. Frontend: Public Usage Page

- [x] 5.1 Create `src/pages/PublicUsagePage.vue` with key input form (shown when no key in route params)
- [x] 5.2 Add API call to `GET /api/public/usage?key=...` using axios — handle 200, 403, 429 responses
- [x] 5.3 Display key name and group name header section
- [x] 5.4 Display subscription cards — active first, inactive dimmed (reduced opacity), hourly_reset shows "Resets in Xh Ym" countdown, progress bar for cost_used/cost_limit_usd
- [x] 5.5 Display token usage table with columns: Model, Input, Output, Cache Creation, Cache Read, Requests, Cost ($)
- [x] 5.6 Handle empty states: "No subscriptions" and "No usage data" messages
- [x] 5.7 Handle error states: invalid key message, rate limit message, loading spinner ← (verify: all component states work — loading, error, empty, success; subscriptions sorted active-first with inactive dimmed)

## 6. Validation

- [x] 6.1 Run `just check` — fix all type-check and lint errors for both frontend and backend ← (verify: `just check` passes with zero errors)
