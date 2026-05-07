## Context

Bonus subscriptions are external API endpoints (e.g., personal Claude accounts) attached to a sub-key. When a request comes in, the proxy tries each bonus server in sequence before falling back to group servers. Today the trial order is `created_at ASC`, fixed forever at insert time. Admins cannot demote a legacy bonus or promote a newly-added primary bonus without deleting and recreating the row, which loses usage history.

This change introduces an explicit sort order controlled by the admin.

## Goals / Non-Goals

**Goals:**
- Admin can reorder bonus subscriptions attached to a sub-key via ↑/↓ buttons.
- Display order (admin table, public usage cards) matches proxy trial order.
- Existing installations continue to behave as before until an admin explicitly reorders.
- Reorder is atomic — either all positions update or none.

**Non-Goals:**
- Reordering non-bonus subscriptions through the UI (the schema allows it, but no UI is added).
- Drag-and-drop reordering.
- Reordering UI on `PublicUsagePage.vue` (end users are read-only).
- Changing the tiebreak behavior of `weekly_cost_limit_usd`, `status`, or any other ordering dimension outside the bonus list.

## Decisions

### Decision: Single `sort_order` column on `key_subscriptions` (applies to all sub types)
**Rationale:** One column used by every query is simpler than a bonus-only side table. Default `0` means existing rows behave identically (tiebreak by `created_at`). Non-bonus subs are unaffected because no UI writes their `sort_order` and their ORDER BY clauses keep `created_at` as the primary sort dimension.

**Alternatives considered:**
- Separate `bonus_sort_order` column → adds a nullable column with narrower semantics; unnecessary.
- `ORDER BY (SELECT...)` with a side table → over-engineered for a single integer.

### Decision: Full-list replace API (`PUT /reorder` with `{ ordered_ids: [...] }`)
**Rationale:** Client sends the exact state it wants. The server validates the set matches the key's bonus subs, assigns `sort_order = index` in a transaction, then invalidates cache. No race between two concurrent single-row updates.

**Alternatives considered:**
- `{ direction: "up"|"down" }` per-row → two concurrent clicks can land in inverted order.
- Per-row `sort_order` PATCH → same race, plus client must pick non-colliding values.

### Decision: ORDER BY `sort_order ASC, created_at ASC` in routing path; `(sub_type='bonus') DESC, sort_order ASC, created_at DESC` in admin list
**Rationale:**
- Routing (`load_subscriptions`) needs total order for bonus trial. All subs with `sort_order = 0` keep `created_at ASC` tiebreak → existing bonus rows keep FIFO.
- Admin list shows bonus rows first (so the reorder buttons are visible at the top) grouped by `sort_order ASC`; non-bonus rows keep `created_at DESC` as today.

### Decision: Reorder includes cancelled bonuses
**Rationale:** Cancelled bonuses may be re-activated later. Keeping them in the ordered list means the admin's intended priority is preserved across lifecycle changes. The API validates the client sent every bonus id for the key, cancelled or not.

### Decision: ↑/↓ buttons (click calls API immediately) instead of drag-and-drop
**Rationale:** Bonus lists are typically ≤5 rows. Buttons are trivial to implement with Quasar's existing `q-btn` and require no new dependencies. Drag-and-drop would need a library and custom row rendering.

## Risks / Trade-offs

- **Risk:** An admin reorders while another request is mid-flight through the proxy. → **Mitigation:** Request in flight has already read its bonus server list; reorder only affects subsequent requests. Redis cache invalidation happens after DB commit so readers either see old or new state, never inconsistent.
- **Risk:** Concurrent reorder requests from two admins. → **Mitigation:** PostgreSQL `SERIALIZABLE` is not required — a simple transaction with `SELECT ... FOR UPDATE` on the affected rows is sufficient to linearize. The set is small and per-key.
- **Risk:** Frontend and backend disagree on which rows are "bonus" after validation. → **Mitigation:** Server validates `ordered_ids` equals the set of bonus sub ids for the key (cancelled included). Any mismatch → 400.
- **Trade-off:** Non-bonus subs also carry `sort_order`, but no UI writes it. Accepted — keeps schema simple and leaves room for future reorder UI on non-bonus.

## Migration Plan

1. Apply migration `049_add_sort_order_to_key_subscriptions.sql` — adds column with default 0. Existing rows take default atomically.
2. Deploy backend. Old routing behaviour preserved (all rows have `sort_order = 0`, tiebreak by `created_at`).
3. Deploy frontend with reorder controls.

No rollback beyond dropping the column is needed; the feature is additive.

## Open Questions

None — all decisions locked during exploration.
