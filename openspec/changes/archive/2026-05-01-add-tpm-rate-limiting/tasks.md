## 1. Database and Models

- [x] 1.1 Add migration `viber-router-api/migrations/044_add_tpm_limit.sql` with nullable `tpm_limit FLOAT8` columns on `subscription_plans` and `key_subscriptions`
- [x] 1.2 Add `tpm_limit: Option<f64>` to backend subscription plan create/update/response models
- [x] 1.3 Add `tpm_limit: Option<f64>` to backend key subscription models and query mappings ← (verify: existing records with NULL TPM deserialize correctly and migration is backward compatible)

## 2. Admin Plan and Assignment APIs

- [x] 2.1 Include `tpm_limit` in subscription plan INSERT, UPDATE, SELECT, and validation paths
- [x] 2.2 Auto-sync `tpm_limit` to active subscriptions when a plan update changes or clears TPM
- [x] 2.3 Add POST `/api/admin/subscription-plans/{id}/sync-tpm` to copy plan TPM to active subscriptions and invalidate affected caches
- [x] 2.4 Copy `tpm_limit` from plans when assigning subscriptions through key subscription admin routes
- [x] 2.5 Copy `tpm_limit` from plans when auto-assigning subscriptions through group key routes ← (verify: manual assignment, auto-assignment, update auto-sync, and sync endpoint all preserve nullable TPM semantics)

## 3. Subscription Selection and TPM Enforcement

- [x] 3.1 Extend `SubCheckResult::Allowed` and bonus fallback subscription metadata to carry `tpm_limit: Option<f64>` without applying TPM to bonus subscriptions
- [x] 3.2 Pass subscription `tpm_limit` through `check_subscriptions()` for fixed, hourly_reset, and pay_per_request subscriptions
- [x] 3.3 Implement `is_tpm_limited()` using Redis key `sub_tpm:{subscription_id}`, current counter comparison, TTL lookup, and fail-open error handling
- [x] 3.4 Implement `wait_for_tpm()` with asynchronous sleep until Redis TTL reset and a maximum of five retries before returning a TPM 429 condition
- [x] 3.5 Call `wait_for_tpm()` in `proxy.rs` after `check_subscriptions()` returns `Allowed` and before the upstream server waterfall ← (verify: TPM-limited selected subscriptions wait instead of falling through, Redis failures allow requests, and bonus routing bypasses TPM)

## 4. TPM Token Accounting

- [x] 4.1 Implement `increment_tpm()` with Redis `INCRBY` by actual `input_tokens + output_tokens` and `EXPIRE 60` only when the returned total equals the added amount
- [x] 4.2 Call `increment_tpm()` after non-streaming billing responses complete with parsed token usage
- [x] 4.3 Call `increment_tpm()` after SSE billing streams complete with parsed token usage
- [x] 4.4 Ensure responses with missing token counts do not use estimates for TPM accounting ← (verify: counter TTL is not extended for existing windows and both streaming/non-streaming responses increment only after actual usage is parsed)

## 5. Public API and Frontend

- [x] 5.1 Include `tpm_limit` in public usage subscription response structs and queries
- [x] 5.2 Add `tpm_limit` to `PlansPage.vue` plan interface, form state, create/update payloads, edit population, reset behavior, and TPM limit input field
- [x] 5.3 Add TPM Limit column and sync TPM action to `PlansPage.vue`
- [x] 5.4 Add `tpm_limit` to `GroupDetailPage.vue` subscription table data and columns ← (verify: plans and group detail tables display nullable TPM consistently and sync TPM calls the correct endpoint)

## 6. Validation and Checks

- [x] 6.1 Add or update backend tests for migration/model mapping, plan create/update/sync, subscription assignment, TPM wait behavior, fail-open behavior, and token counter increments
- [x] 6.2 Add or update frontend tests or manual verification notes for TPM form/table/sync UI behavior
- [x] 6.3 Run `just check` from the repository root and fix all reported type, lint, and formatting errors ← (verify: full frontend and backend checks pass before implementation is considered complete)
