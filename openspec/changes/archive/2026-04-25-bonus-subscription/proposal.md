## Why

Sub-keys currently rely entirely on the group's shared servers, giving no way to attach a dedicated upstream (e.g., a Claude Max seat) to a specific key without creating a separate group. Adding a `bonus` subscription type lets admins bind an external server directly to a sub-key so it is tried first, with automatic fallback to the group's normal servers when the bonus server is unavailable.

## What Changes

- New `bonus` value added to the `sub_type` CHECK constraint on `key_subscriptions` and `subscription_plans`.
- Five new nullable columns added to `key_subscriptions`: `bonus_base_url`, `bonus_api_key`, `bonus_name`, `bonus_quota_url`, `bonus_quota_headers`.
- `KeySubscription` Rust model extended with the five new optional fields.
- Admin POST endpoint for subscriptions extended to accept direct bonus creation (no `plan_id` required).
- Subscription engine `SubCheckResult` gains a new `BonusServers` variant; `check_subscriptions()` separates bonus from non-bonus subs and emits this variant when bonus subs are active.
- Proxy engine handles `BonusServers`: tries each bonus server in FIFO order before the group server waterfall, falls back gracefully on non-2xx responses.
- Public usage API `PublicSubscription` extended with `bonus_name`, `bonus_quotas` (fetched from optional quota URL), and `bonus_usage` (per-model request counts).
- Admin UI (`GroupDetailPage.vue`) gains an "Add Bonus" button and dialog; subscription table renders bonus rows with name/type-only display.
- Public usage UI (`PublicUsagePage.vue`) renders a distinct card for bonus subscriptions with quota progress bars and per-model request counts.
- `useSubscriptionType` composable updated with `bonus` label, tooltip, and option entry.

## Capabilities

### New Capabilities

- `bonus-subscription`: End-to-end bonus subscription type — creation, proxy routing, quota display, and admin/public UI rendering.

### Modified Capabilities

- `key-subscriptions`: New `sub_type` value `bonus` and five new nullable columns change the data model and creation contract.
- `subscription-enforcement`: `SubCheckResult` gains the `BonusServers` variant, changing how the proxy engine consumes check results.
- `proxy-engine`: Bonus server waterfall is a new routing step inserted before the existing group server waterfall.
- `public-usage-api`: `PublicSubscription` response shape extended with bonus-specific fields.
- `public-usage-page`: New card style and quota progress bars for bonus subscriptions.
- `subscription-keys-ui`: Admin subscription table and creation flow updated to support bonus type.

## Impact

- **Database**: One migration file; alters `key_subscriptions` column set and updates CHECK constraints on both `key_subscriptions` and `subscription_plans`.
- **Backend API**: `POST /api/admin/groups/:group_id/keys/:key_id/subscriptions` accepts bonus payload without `plan_id`; `GET` public usage response shape changes.
- **Rust model**: `KeySubscription` struct in `viber-router-api/src/models/key_subscription.rs` gains five `Option` fields; all match arms on `SubCheckResult` must handle the new `BonusServers` variant.
- **Proxy hot path**: A new upstream HTTP call per bonus server is added before the existing failover waterfall — no cost is tracked for bonus requests.
- **Frontend**: `GroupDetailPage.vue`, `PublicUsagePage.vue`, `useSubscriptionType.ts` all require changes; no new routes or stores needed.
- **External dependency**: Backend makes outbound HTTP GET to admin-supplied `bonus_quota_url` with a 5-second timeout; failures are silent (empty quota array returned).
