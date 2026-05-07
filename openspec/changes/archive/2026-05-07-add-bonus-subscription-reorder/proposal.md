## Why

A sub-key can have multiple bonus subscriptions (e.g., several Claude Code Max accounts). Currently their order is fixed to `created_at ASC`, which determines both the admin table display order and the FIFO order in which the proxy tries each bonus server. Admins have no way to prioritize one bonus over another without deleting and recreating them. This blocks workflows like "try my primary account first, fall back to the secondary" when the primary was added second.

## What Changes

- Add a `sort_order` column on `key_subscriptions` so bonus subscriptions can be ordered independently of creation time.
- Proxy trial order for bonus servers becomes `sort_order ASC, created_at ASC` (tiebreak preserves existing behavior for rows with default `sort_order = 0`).
- New admin endpoint `PUT /api/admin/groups/:group_id/keys/:key_id/subscriptions/reorder` that takes `{ ordered_ids: [uuid, ...] }` containing the full set of bonus subscription ids for that key and atomically rewrites their `sort_order` values.
- Admin UI (`GroupDetailPage.vue`) gains ↑/↓ buttons on each bonus row. Clicking swaps that bonus with its neighbor and calls the reorder endpoint.
- Admin subscription table lists bonus rows first (ordered by `sort_order ASC`), then non-bonus rows (ordered by `created_at DESC` as today).
- Public usage page bonus cards are rendered in `sort_order ASC` order (read-only — no reorder UI for end users).
- Redis cache `key_subs:{key_id}` is invalidated after a successful reorder.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `bonus-subscription`: Adds requirements for sort order storage, ordered proxy trial, reorder endpoint, ordered rendering in admin and public UIs.

## Impact

- **Database**: New migration `049_add_sort_order_to_key_subscriptions.sql` adds `sort_order INTEGER NOT NULL DEFAULT 0` to `key_subscriptions`. Safe for existing rows (default 0 tiebreaks by `created_at`, preserving current order).
- **Backend**:
  - `viber-router-api/src/models/key_subscription.rs` (`KeySubscription` struct + test fixtures)
  - `viber-router-api/src/routes/admin/key_subscriptions.rs` (new `reorder_subscriptions` handler, route registration, updated `list_subscriptions` ORDER BY)
  - `viber-router-api/src/subscription.rs` (`load_subscriptions` ORDER BY)
  - `viber-router-api/src/routes/public/usage.rs` (bonus listing ORDER BY)
- **Frontend**: `src/pages/GroupDetailPage.vue` gains reorder controls on bonus rows.
- **No behavior change for non-bonus subscriptions** — all existing rows get `sort_order = 0` and routing/listing tiebreaks by `created_at` as before.
- **No UI change on `PublicUsagePage.vue`** — ordering is driven entirely by the backend query.
