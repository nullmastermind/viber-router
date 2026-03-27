## 1. Database Migration

- [x] 1.1 Create `viber-router-api/migrations/024_add_rate_limit_to_group_servers.sql` with two ALTER TABLE statements adding `max_requests INTEGER` and `rate_window_seconds INTEGER` columns (nullable, no default) ← (verify: migration runs without errors, columns exist in group_servers table)

## 2. Backend Models

- [x] 2.1 Add `max_requests: Option<i32>` and `rate_window_seconds: Option<i32>` to `GroupServer` struct in `models/group_server.rs`
- [x] 2.2 Add `max_requests: Option<i32>` and `rate_window_seconds: Option<i32>` to `GroupServerDetail` struct
- [x] 2.3 Add `max_requests: Option<i32>` and `rate_window_seconds: Option<i32>` to `AdminGroupServerDetail` struct
- [x] 2.4 Add `max_requests: Option<Option<i32>>` and `rate_window_seconds: Option<Option<i32>>` to `UpdateAssignment` struct ← (verify: all 4 structs have the new fields, types match design)

## 3. Backend Admin API

- [x] 3.1 Add `validate_rate_limit_fields` function in `routes/admin/group_servers.rs` — all-or-nothing validation, values >= 1, mirroring `validate_cb_fields`
- [x] 3.2 Call `validate_rate_limit_fields` in `update_assignment` handler
- [x] 3.3 Add CASE WHEN clauses for `max_requests` and `rate_window_seconds` to the UPDATE query in `update_assignment`, with corresponding bind parameters
- [x] 3.4 Append `gs.max_requests, gs.rate_window_seconds` to the SELECT query in `get_group` handler (`routes/admin/groups.rs`) ← (verify: admin API accepts rate limit fields, validates correctly, returns them in group detail)

## 4. Rate Limiter Module

- [x] 4.1 Create `viber-router-api/src/rate_limiter.rs` with `is_rate_limited(redis, group_id, server_id, max_requests, window_seconds) -> bool` function — GET counter, return true if >= max_requests, fail open on Redis error
- [x] 4.2 Add `increment_rate_limit(redis, group_id, server_id, window_seconds)` function — INCR counter, set TTL if count == 1, silently skip on Redis error
- [x] 4.3 Register `mod rate_limiter` in `main.rs` ← (verify: module compiles, functions have correct signatures matching design)

## 5. Proxy Integration

- [x] 5.1 Append `gs.max_requests, gs.rate_window_seconds` to the SELECT query in `resolve_group_config` (`routes/proxy.rs`)
- [x] 5.2 Add rate limit check in the failover waterfall loop — after circuit breaker check, before `any_server_attempted = true`: if `max_requests` is Some, call `is_rate_limited`; if true, set `any_rate_limited = true` flag and `continue`
- [x] 5.3 After rate limit check passes, call `increment_rate_limit` before sending the request
- [x] 5.4 After the server loop, if `!any_server_attempted && any_rate_limited`, return 429 with `rate_limit_error` type and "Rate limit exceeded" message ← (verify: proxy skips rate-limited servers, increments counter, returns correct 429 when all rate-limited)

## 6. Frontend Store

- [x] 6.1 Add `max_requests: number | null` and `rate_window_seconds: number | null` to `GroupServerDetail` interface in `stores/groups.ts`
- [x] 6.2 Add `max_requests?: number | null` and `rate_window_seconds?: number | null` to `updateAssignment` input type ← (verify: TypeScript types match backend API)

## 7. Frontend UI

- [x] 7.1 Add rate limit badge on server list in `GroupDetailPage.vue` — show `{max_requests}/{rate_window_seconds}s` when both fields are non-null
- [x] 7.2 Add `editServerRlForm` reactive ref with `max_requests` and `rate_window_seconds` fields
- [x] 7.3 Add Rate Limit section in Edit Server dialog below Circuit Breaker — two `q-input` number fields: "Max Requests" and "Window (seconds)"
- [x] 7.4 Populate rate limit fields in `openEditServer` function from server data
- [x] 7.5 Include rate limit fields in `saveEditServer` function's `updateAssignment` call ← (verify: badge displays correctly, edit dialog shows/saves rate limit config, `just check` passes)

## 8. Final Verification

- [x] 8.1 Run `just check` — fix all type errors and lint errors ← (verify: zero errors from type-check and lint for both frontend and backend)
