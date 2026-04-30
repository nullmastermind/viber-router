## 1. Backend Update API

- [x] 1.1 Add `UpdateBonusSubscription` with `bonus_allowed_models: Option<Vec<String>>` in `/root/projects/viber-router/viber-router-api/src/models/key_subscription.rs`
- [x] 1.2 Add a focused handler for updating bonus allowed models in `/root/projects/viber-router/viber-router-api/src/routes/admin/key_subscriptions.rs`
- [x] 1.3 Validate that the target subscription exists, belongs to the requested key, has `sub_type = 'bonus'`, and is active before updating
- [x] 1.4 Persist the updated `bonus_allowed_models`, invalidate Redis cache key `key_subs:{key_id}`, and return the updated subscription
- [x] 1.5 Register `PUT /{sub_id}/bonus-allowed-models` in the key subscription admin router ← (verify: route is reachable, rejects ineligible subscriptions, invalidates Redis only after successful update, and returns the updated subscription)

## 2. Frontend Inline Editor

- [x] 2.1 Add state for the bonus allowed models editor dialog or popup in `/root/projects/viber-router/src/pages/GroupDetailPage.vue`
- [x] 2.2 Make the allowed models cell editable only when `sub_type === 'bonus'` and the subscription is active
- [x] 2.3 Populate the editor with current subscription values and options from the existing `bonusAllowedModelOptions` computed source
- [x] 2.4 Save changes through the new API endpoint using `api.put` with `{ bonus_allowed_models: string[] }`
- [x] 2.5 Show success and error notifications using the existing `$q.notify` pattern and refresh subscriptions after successful save ← (verify: active bonus rows can be edited end-to-end, cleared selections display as "All models", and non-bonus or inactive rows do not expose editing)

## 3. Validation

- [x] 3.1 Run backend checks for the changed Rust code and fix any formatting, type, or clippy issues
- [x] 3.2 Run frontend checks for the changed Vue/TypeScript code and fix any Biome or type issues
- [x] 3.3 Run `just check` from `/root/projects/viber-router` and fix all reported errors ← (verify: full repository check passes without skipped steps)
