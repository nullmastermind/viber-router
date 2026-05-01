## Why

Subscriptions currently support RPM limiting, but they do not protect upstream capacity from high-token bursts within a minute. Adding TPM limiting lets admins cap token throughput per subscription while preserving user experience by waiting for the fixed window to reset instead of immediately skipping or rejecting eligible subscriptions.

## What Changes

- Add optional `tpm_limit` values to subscription plan templates and key subscription snapshots.
- Enforce TPM for active non-bonus subscriptions on billing proxy requests using a Redis fixed-window counter per subscription.
- Make TPM enforcement wait for the current 60-second window to reset, retrying up to five times before returning 429.
- Count actual input plus output tokens after the upstream response completes and add them to the subscription TPM counter.
- Fail open when Redis is unavailable for TPM checks or increments.
- Copy `tpm_limit` from plans to newly assigned subscriptions, auto-sync it to active subscriptions when plans change, and provide an admin sync endpoint.
- Expose `tpm_limit` in admin plan/subscription UI surfaces and in the public usage API.

## Capabilities

### New Capabilities
- `subscription-tpm-rate-limiting`: Covers subscription-level tokens-per-minute limits, wait behavior, Redis counters, token accounting, and admin/public API exposure.

### Modified Capabilities
- `subscription-plans`: Plan records and admin APIs include optional TPM limits and syncing behavior.
- `key-subscriptions`: Subscription snapshots include optional TPM limits copied from assigned plans.
- `subscription-enforcement`: Proxy subscription enforcement waits on TPM windows for non-bonus subscriptions.
- `public-usage-api`: Public subscription usage responses include TPM limits.
- `subscription-plans-ui`: Plans UI supports displaying, editing, and syncing TPM limits.
- `subscription-keys-ui`: Subscription tables for group keys display TPM limits.

## Impact

- Database migration adds nullable `tpm_limit FLOAT8` columns to `subscription_plans` and `key_subscriptions`.
- Backend models, admin routes, assignment flows, public usage response structs, and proxy subscription enforcement are updated.
- Redis gains `sub_tpm:{subscription_id}` fixed-window counters with 60-second TTLs.
- Frontend plan and group detail pages gain TPM fields, columns, and sync controls.
- No breaking API changes are expected because `tpm_limit` is optional and nullable.
