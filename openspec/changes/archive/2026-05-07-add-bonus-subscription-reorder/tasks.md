## 1. Database

- [x] 1.1 Create `viber-router-api/migrations/049_add_sort_order_to_key_subscriptions.sql` with `ALTER TABLE key_subscriptions ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0`
- [x] 1.2 Verify migration applies cleanly against existing data (default 0 for existing rows) <- (verify: migration file exists, is numbered 049, and matches design.md Migration Plan)

## 2. Backend Model

- [x] 2.1 Add `pub sort_order: i32` to `KeySubscription` struct in `viber-router-api/src/models/key_subscription.rs`
- [x] 2.2 Update existing unit test fixtures in `key_subscription.rs` to include `sort_order: 0` in JSON payloads <- (verify: `cargo check` and `cargo test -p viber-router-api --lib models::key_subscription` pass)

## 3. Backend Reorder Endpoint

- [x] 3.1 Add `ReorderSubscriptions { ordered_ids: Vec<Uuid> }` request struct (colocated in `key_subscriptions.rs` or `models/key_subscription.rs`)
- [x] 3.2 Implement `reorder_subscriptions` handler in `viber-router-api/src/routes/admin/key_subscriptions.rs`:
  - Load all bonus sub ids for `:key_id`
  - Validate `ordered_ids` has no duplicates, has same length as bonus set, and every id is a bonus sub for this key
  - Return 400 with clear error on any validation failure
  - Open transaction, `UPDATE key_subscriptions SET sort_order = $1 WHERE id = $2` for each id at its index, commit
  - Invalidate Redis key `key_subs:{key_id}` after commit
  - Return 200 with `{ "ok": true }`
- [x] 3.3 Register route `.route("/reorder", put(reorder_subscriptions))` in `key_subscriptions.rs` router
- [x] 3.4 Update `list_subscriptions` SQL ORDER BY to `(sub_type = 'bonus') DESC, sort_order ASC, created_at DESC`
- [x] 3.5 Add unit tests in `key_subscriptions.rs`:
  - `test_reorder_validation_rejects_duplicate_ids` (pure validation logic -- may require extracting a pure function)
  - `test_reorder_validation_rejects_missing_ids`
  - `test_reorder_validation_rejects_extra_ids`
  - `test_reorder_validation_rejects_non_bonus_ids`
  - `test_reorder_validation_rejects_foreign_key_ids`
  - `test_reorder_validation_accepts_correct_full_set` <- (verify: all 6 tests pass, validation covers every scenario in spec delta Requirement: Reorder bonus subscriptions endpoint)

## 4. Backend Routing Order

- [x] 4.1 In `viber-router-api/src/subscription.rs` (`load_subscriptions`), change `ORDER BY created_at ASC` to `ORDER BY sort_order ASC, created_at ASC`
- [x] 4.2 In `viber-router-api/src/routes/public/usage.rs` (bonus listing query around line 146), change ORDER BY to `status = 'active' DESC, (sub_type = 'bonus') DESC, sort_order ASC, created_at DESC` <- (verify: all three ORDER BY changes preserve existing behavior when sort_order is 0 for all rows, and route bonus servers in admin-defined order otherwise -- maps to spec delta Requirement: Bonus server list in SubCheckResult and Requirement: Public usage bonus rendering order)

## 5. Frontend

- [x] 5.1 In `src/pages/GroupDetailPage.vue`, add up/down buttons for bonus rows (inside actions column or a new dedicated column rendered only for `sub_type === 'bonus'`)
- [x] 5.2 Implement `onMoveBonus(keyId, subId, direction: 'up' | 'down')`:
  - Compute current bonus list for `keyId` (filter `sub_type === 'bonus'` from `keySubscriptions[keyId].data`)
  - Determine new ordered ids by swapping target with its neighbor
  - Set loading state on that row's buttons
  - `await $api.put('/api/admin/groups/:gid/keys/:kid/subscriptions/reorder', { ordered_ids })`
  - On success: `await loadKeySubscriptions(keyId, ...)` to refetch
  - On error: show negative toast + refetch to revert
  - Clear loading state
- [x] 5.3 Disable up button on first bonus row, down button on last bonus row
- [x] 5.4 Ensure button click does not toggle row expansion (`@click.stop`) and uses appropriate icons (`arrow_upward` / `arrow_downward`) <- (verify: UI flow matches spec delta Requirement: Admin UI reorder controls on bonus rows -- buttons render, disabled states correct, error path refetches and toasts)

## 6. Checks

- [x] 6.1 Run `just check` from workspace root; fix any lint/type errors
- [x] 6.2 Run `cargo test -p viber-router-api --lib` and confirm new reorder tests pass alongside existing tests
- [ ] 6.3 Manual verify (document in change): apply migration, restart API, create 2-3 bonus subs on a test key, click up/down in admin UI, refresh page, confirm persisted order matches last reorder; confirm public usage page shows bonus cards in the same order <- (verify: every `Scenario:` in the spec delta has a concrete manual or automated check covering it)
