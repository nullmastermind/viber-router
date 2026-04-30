## Why

Bonus subscriptions currently accept every model routed through a group, which prevents admins from using bonus capacity only for specific model families or tiers. Adding a per-bonus model allowlist gives admins finer routing control while preserving the current accept-all behavior for existing subscriptions.

## What Changes

- Add an optional `bonus_allowed_models` array field to bonus subscriptions.
- Allow admins to select zero or more allowed model names when creating a bonus subscription.
- Persist selected model names on bonus subscription creation.
- Filter bonus subscription servers during proxy subscription checks so restricted bonus subscriptions only handle matching request models.
- Display each bonus subscription's allowed models in the group detail UI, showing "All models" for null or empty allowlists.
- Preserve backward compatibility: null or empty allowlists continue to accept all models.

## Capabilities

### New Capabilities

- `bonus-subscription-model-allowlist`: Admin-configurable model allowlists for bonus subscriptions and proxy enforcement of those allowlists.

### Modified Capabilities

- `bonus-subscription`: Bonus subscriptions gain optional model-specific routing behavior while retaining accept-all semantics by default.

## Impact

- Database: `key_subscriptions` gains a nullable `bonus_allowed_models TEXT[]` column.
- Backend models: `KeySubscription` and `AssignSubscription` include the optional allowed-model list.
- Backend admin API: bonus subscription creation accepts and stores `bonus_allowed_models`.
- Backend proxy routing: bonus subscription eligibility considers request model allowlists.
- Frontend admin UI: Add Bonus dialog sends selected model names; bonus rows display configured allowlists.
