## Why

Bonus subscriptions can be created with model allowlists, but administrators cannot change `bonus_allowed_models` after creation. This makes mistakes or later group model changes hard to correct without canceling and recreating the subscription.

## What Changes

- Add backend support for updating `bonus_allowed_models` on an existing key subscription.
- Expose an admin API endpoint that updates only the bonus allowed model list for active bonus subscriptions belonging to the requested key.
- Add frontend inline editing for the bonus allowed models table cell in the group detail subscription table.
- Preserve the existing Add Bonus dialog flow and avoid editing any other bonus subscription fields.
- Treat an empty model list as unrestricted/all models, consistent with the current null behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `bonus-subscription-model-allowlist`: Allow administrators to edit the model allowlist of an active bonus subscription after creation.

## Impact

- Backend model: `/root/projects/viber-router/viber-router-api/src/models/key_subscription.rs`
- Backend route: `/root/projects/viber-router/viber-router-api/src/routes/admin/key_subscriptions.rs`
- Frontend page: `/root/projects/viber-router/src/pages/GroupDetailPage.vue`
- API surface: new or extended admin subscription update endpoint for bonus allowed models.
- Cache behavior: updates must invalidate the Redis key subscription cache for the affected key.
- Validation: implementation must continue to pass `just check`, including Biome and Cargo clippy.
