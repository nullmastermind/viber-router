## 1. Database and Backend Data Model

- [x] 1.1 Add a new backend migration under `/root/projects/viber-router/viber-router-api/migrations/` that runs `ALTER TABLE key_subscriptions ADD COLUMN IF NOT EXISTS bonus_allowed_models TEXT[]`
- [x] 1.2 Add `bonus_allowed_models: Option<Vec<String>>` to `KeySubscription` and `AssignSubscription` in `/root/projects/viber-router/viber-router-api/src/models/key_subscription.rs`
- [x] 1.3 Update key subscription SELECT/serialization paths so bonus subscriptions include `bonus_allowed_models` in API responses ← (verify: Rust models compile and admin UI responses expose null, empty, and populated allowlists correctly)

## 2. Admin API Creation Flow

- [x] 2.1 Update the bonus creation branch in `/root/projects/viber-router/viber-router-api/src/routes/admin/key_subscriptions.rs` to include `bonus_allowed_models` in the INSERT column list and bind parameters
- [x] 2.2 Preserve existing validation for required bonus fields while allowing `bonus_allowed_models` to be omitted, null, or an empty array
- [x] 2.3 Confirm normal plan subscription creation remains unchanged and bonus creation returns the stored allowlist ← (verify: POST bonus creation works with omitted, empty, and populated `bonus_allowed_models` without changing normal subscription behavior)

## 3. Subscription Engine Filtering

- [x] 3.1 Add `allowed_models: Vec<String>` to `BonusServer` in `/root/projects/viber-router/viber-router-api/src/subscription.rs`
- [x] 3.2 Update `check_subscriptions()` bonus subscription processing to read each subscription's `bonus_allowed_models`
- [x] 3.3 Filter bonus subscriptions before collecting `BonusServer` entries: include restricted bonuses only when request model matches a stored model name; include null or empty allowlists for all models
- [x] 3.4 Preserve FIFO ordering and fallback subscription behavior for eligible bonus servers ← (verify: restricted non-matching bonus subs are excluded, unrestricted bonus subs remain eligible, and existing fallback behavior still works)

## 4. Admin UI Create Dialog

- [x] 4.1 Add Add Bonus dialog state for selected bonus allowed model names in `/root/projects/viber-router/src/pages/GroupDetailPage.vue`
- [x] 4.2 Add a QSelect multi-select to the Add Bonus dialog using the group's `allowedModels` as options and emitting model `name` strings
- [x] 4.3 Include `bonus_allowed_models` as an array of selected model names when submitting the Add Bonus request, and reset the field when the dialog closes or after successful creation ← (verify: request payload contains model names, not IDs, and empty selection preserves all-model behavior)

## 5. Admin UI Display

- [x] 5.1 Update the frontend subscription type/interface in `/root/projects/viber-router/src/pages/GroupDetailPage.vue` to include optional `bonus_allowed_models`
- [x] 5.2 Add bonus row rendering for allowed models, using chips or comma-separated model names for populated allowlists
- [x] 5.3 Display "All models" when `bonus_allowed_models` is null, undefined, or empty ← (verify: bonus rows clearly distinguish restricted model lists from unrestricted all-model subscriptions)

## 6. Verification

- [x] 6.1 Run backend checks and fix any Rust compile or clippy errors related to sqlx projections, struct fields, or filtering logic
- [x] 6.2 Run frontend checks and fix any TypeScript, template, or Biome errors in `GroupDetailPage.vue`
- [x] 6.3 Run `/root/projects/viber-router/just check` from the workspace root and fix all reported issues ← (verify: full project check passes after backend and frontend changes)
