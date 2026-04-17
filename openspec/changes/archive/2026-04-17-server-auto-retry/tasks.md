## 1. Database Migration

- [x] 1.1 Create `viber-router-api/migrations/040_group_servers_retry.sql` adding `retry_status_codes INTEGER[] DEFAULT NULL`, `retry_count INTEGER DEFAULT NULL`, and `retry_delay_seconds DOUBLE PRECISION DEFAULT NULL` to `group_servers`
- [ ] 1.2 Run migration against local dev database and verify columns exist with correct types and defaults ← (verify: migration applies cleanly, existing rows have all three columns as NULL)

## 2. Backend Models

- [x] 2.1 Add `retry_status_codes: Option<Vec<i32>>`, `retry_count: Option<i32>`, and `retry_delay_seconds: Option<f64>` fields (with `#[serde(default)]`) to `GroupServer`, `GroupServerDetail`, and `AdminGroupServerDetail` structs in `viber-router-api/src/models/group_server.rs`
- [x] 2.2 Add the same three fields (using double-Option: `Option<Option<Vec<i32>>>`, `Option<Option<i32>>`, `Option<Option<f64>>`) to `UpdateAssignment` struct in the same file ← (verify: `cargo check` passes, all struct fields compile correctly)

## 3. Admin Handler — Update Assignment

- [x] 3.1 Add retry fields to the SELECT query in `get_assignment` (or wherever `AdminGroupServerDetail` is fetched) in `viber-router-api/src/routes/admin/group_servers.rs`
- [x] 3.2 Add all-or-nothing validation for the three retry fields in `update_assignment`: reject with HTTP 400 if only some are provided; validate `retry_count >= 1`, `retry_delay_seconds > 0`, each status code in 400–599, and `retry_status_codes` non-empty when set
- [x] 3.3 Add retry fields to the UPDATE SQL in `update_assignment` using the `CASE WHEN $N THEN $M ELSE column END` pattern for optional updates ← (verify: `cargo clippy -- -D warnings` passes; PUT with valid retry config updates DB; PUT with partial config returns 400; PUT omitting retry fields leaves existing values unchanged)

## 4. Proxy — Server SELECT and Retry Loop

- [x] 4.1 Add `retry_status_codes`, `retry_count`, and `retry_delay_seconds` to the server SELECT query in `viber-router-api/src/routes/proxy.rs` (the query that populates `GroupServerDetail` for the proxy loop)
- [x] 4.2 In the non-streaming failover check point (around line 1242), add retry logic: if server has retry config and response status is in `retry_status_codes`, loop up to `retry_count` times sleeping `retry_delay_seconds` via `tokio::time::sleep` before each retry; break early if a retry returns a status not in `retry_status_codes`; after loop exhaustion fall through to existing failover check
- [x] 4.3 Apply the same retry logic to the streaming failover check point in `proxy.rs` ← (verify: `cargo clippy -- -D warnings` passes; proxy retries same server on matching status; proxy fails over after retries exhausted; proxy skips retry when config is NULL)

## 5. Frontend Store

- [x] 5.1 Add `retry_status_codes: number[] | null`, `retry_count: number | null`, and `retry_delay_seconds: number | null` to the `GroupServerDetail` interface in `src/stores/groups.ts`
- [x] 5.2 Add the same three fields to the `updateAssignment` input type/payload in `src/stores/groups.ts` ← (verify: `bun run build` or type-check passes with no TypeScript errors)

## 6. Frontend UI

- [x] 6.1 Add a `replay` icon button to each server row in `src/pages/GroupDetailPage.vue` (next to the existing `tune` button), following the same pattern
- [x] 6.2 Add reactive state for the retry config dialog (`showRetryDialog`, `editRetryServer`, `retryForm` with the three fields) in `GroupDetailPage.vue`
- [x] 6.3 Implement the retry config `q-dialog` with inputs for `retry_status_codes` (comma-separated or chip input), `retry_count` (number), and `retry_delay_seconds` (number), pre-populated from the server's current config
- [x] 6.4 Implement `saveRetryConfig` method that calls `updateAssignment` with the three fields (or all-null to clear) and closes the dialog on success
- [x] 6.5 Add a badge or chip on the server row that is visible when all three retry fields are non-null, consistent with the rate limit badge pattern ← (verify: `bun run lint` passes; retry button opens dialog; saving valid config updates the row badge; clearing config removes the badge)

## 7. Final Check

- [x] 7.1 Run `just check` (type-check + lint for both frontend and backend) and fix all reported errors ← (verify: `just check` exits with code 0, no lint or type errors in any file)
