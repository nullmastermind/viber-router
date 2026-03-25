## 1. Database Migration

- [x] 1.1 Create migration adding `cb_max_failures INTEGER`, `cb_window_seconds INTEGER`, `cb_cooldown_seconds INTEGER` (all nullable) to `group_servers` table

## 2. Backend Models

- [x] 2.1 Add `cb_max_failures: Option<i32>`, `cb_window_seconds: Option<i32>`, `cb_cooldown_seconds: Option<i32>` to `GroupServer`, `GroupServerDetail`, and `GroupConfig` server structs
- [x] 2.2 Add the three fields as `Option<Option<i32>>` to `UpdateAssignment` struct
- [x] 2.3 Add all-or-nothing validation function: if any of the three CB fields is `Some`, all three must be `Some` and each value >= 1; return 400 on violation ← (verify: partial CB config returns 400, all-null accepted, all-valid accepted, zero values rejected)

## 3. Backend Routes — Assignment

- [x] 3.1 Update `assign_server` INSERT query to include the three CB columns (defaulting to NULL)
- [x] 3.2 Update `update_assignment` handler to COALESCE the three CB fields and call validation before executing query
- [x] 3.3 Update group detail query in `groups.rs` to SELECT the three CB columns in GroupServerDetail ← (verify: admin API returns CB fields for all servers, update correctly persists CB config and invalidates cache)

## 4. Circuit Breaker Redis Logic

- [x] 4.1 Create `src/circuit_breaker.rs` module with functions: `is_circuit_open(redis, group_id, server_id) -> bool`, `record_error(redis, group_id, server_id, max_failures, window_seconds, cooldown_seconds) -> bool (tripped?)`, and `check_re_enabled(redis, group_id, server_id) -> bool (was re-enabled?)`
- [x] 4.2 `is_circuit_open`: check EXISTS `cb:open:{group_id}:{server_id}`
- [x] 4.3 `record_error`: INCR `cb:err:{g}:{s}`, set TTL=window_seconds if new key (TTL == -1), if count >= max_failures then SET `cb:open:{g}:{s}` with TTL=cooldown_seconds and DEL error counter, return whether tripped
- [x] 4.4 `check_re_enabled`: if `cb:open:{g}:{s}` does NOT exist AND `cb:realerted:{g}:{s}` does NOT exist, SET `cb:realerted:{g}:{s}` with TTL=60 and return true; else return false ← (verify: error counting increments correctly, threshold trips breaker, open key has correct TTL, re-enable detection works with marker key)

## 5. Proxy Integration

- [x] 5.1 In proxy failover loop, before sending request: if server has CB configured, call `is_circuit_open`; if open, skip server (continue)
- [x] 5.2 After each error (failover status code, connection error, TTFT timeout): if server has CB configured, call `record_error`; if tripped, spawn Telegram alert task
- [x] 5.3 Before circuit-open check: call `check_re_enabled` for servers that have CB configured; if re-enabled, spawn Telegram re-enable alert task ← (verify: circuit-broken servers are skipped, errors increment counter, threshold trips breaker with alert, re-enable sends alert on next request, servers without CB configured are unaffected)

## 6. Telegram Alerts

- [x] 6.1 Add `send_circuit_breaker_alert` function in `telegram_notifier.rs` — accepts server name, group name, error count, window seconds, cooldown seconds; formats message with circuit breaker emoji and details; uses same delivery mechanism (settings, chat IDs, cooldown key `tg:cooldown:cb:{group_id}:{server_id}`)
- [x] 6.2 Add `send_circuit_re_enable_alert` function — accepts server name, group name; formats re-enable message ← (verify: trip alert includes all details, re-enable alert sent correctly, cooldown prevents duplicate alerts)

## 7. Circuit Status API

- [x] 7.1 Add `GET /api/admin/groups/{group_id}/circuit-status` endpoint: query all servers in group that have CB configured, check `cb:open` key TTL for each, return `[{server_id, is_open, remaining_seconds}]` ← (verify: returns correct open/closed state and remaining TTL for each server, empty array when no CB configured)

## 8. Frontend — Types & Store

- [x] 8.1 Add `cb_max_failures`, `cb_window_seconds`, `cb_cooldown_seconds` to `GroupServerDetail` interface in `stores/groups.ts`
- [x] 8.2 Add `CircuitStatus` interface `{server_id: string, is_open: boolean, remaining_seconds: number}` and `fetchCircuitStatus(groupId)` API call

## 9. Frontend — GroupDetailPage UI

- [x] 9.1 Add circuit breaker fields to edit server dialog: three number inputs (Max Failures, Failure Window seconds, Cooldown seconds) with Vietnamese description explaining behavior; clear all three when any is cleared (all-or-nothing UX)
- [x] 9.2 Add "Circuit Open (Xm Ys)" badge on server row when circuit is open; poll circuit status every 10s when any circuit is open; show countdown updating each poll
- [x] 9.3 Include CB fields in `toggleServerEnabled` / edit server save calls via `updateAssignment` ← (verify: CB fields save and load correctly, badge shows with countdown when circuit is open, badge disappears when circuit closes, edit dialog enforces all-or-nothing)

## 10. Cache Update

- [x] 10.1 Update the proxy server query in `proxy.rs` to SELECT the three CB columns so they are included in `GroupConfig` cached in Redis ← (verify: GroupConfig in Redis includes CB fields, proxy reads CB config from cache correctly)
