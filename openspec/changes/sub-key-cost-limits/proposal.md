## Why

Sub-keys currently have no usage enforcement — once created and active, they can consume unlimited API resources. Administrators need the ability to set cost-based limits on sub-keys through a subscription model, where each sub-key can be assigned one or more subscription plans that control spending by total budget, per-model budget, and time-windowed rate limits. When limits are reached, the proxy must block requests with 429 responses.

## What Changes

- New `subscription_plans` table for global plan templates (fixed or hourly-reset types, with per-model cost limits)
- New `key_subscriptions` table for plan instances assigned to sub-keys (snapshot of plan values at assignment time)
- Proxy enforcement: pre-request cost check via Redis counters, post-request cost calculation and counter update
- Cost calculation in proxy using model pricing × server rate multipliers, stored in `token_usage_logs.cost_usd`
- Redis-based real-time cost tracking with automatic rebuild from DB on cache miss
- New admin API endpoints for subscription plan CRUD and key subscription management
- New "Plans" sidebar page for global subscription plan management
- Subscription management UI in GroupDetailPage expanded sub-key rows
- **BREAKING**: Drop `monthly_token_limit` and `monthly_request_limit` columns from `group_keys` (unused, replaced by subscription system)

## Capabilities

### New Capabilities
- `subscription-plans`: Global subscription plan catalog — CRUD for plan templates with type (fixed/hourly_reset), cost limits, per-model limits, reset hours, and duration
- `key-subscriptions`: Assignment of subscription plan instances to sub-keys — snapshot plan values, track status (active/exhausted/expired/cancelled), activation on first request
- `subscription-enforcement`: Proxy-level cost limit enforcement — pre-request budget check via Redis, post-request cost calculation and counter update, 429 blocking when limits exceeded
- `subscription-cost-tracking`: Real-time cost tracking in Redis with persistent storage in token_usage_logs — includes per-model tracking, window-based tracking for hourly resets, and DB rebuild on cache miss
- `subscription-plans-ui`: Admin UI page for managing global subscription plans with model limits editor
- `subscription-keys-ui`: Subscription management within sub-key expanded rows — view active subscriptions, add from plan dropdown, cancel

### Modified Capabilities
- `token-usage-storage`: Add `cost_usd` and `subscription_id` columns to `token_usage_logs`
- `token-usage-extraction`: Calculate cost at extraction time using cached model pricing × server rates, include in TokenUsageEntry

## Impact

- **Database**: 2 new tables, 2 new columns on existing table, 1 migration to drop unused columns
- **Proxy (proxy.rs)**: New enforcement check after sub-key resolution, cost calculation after successful response, Redis counter updates in both non-streaming and streaming paths
- **Usage buffer (usage_buffer.rs)**: Extended TokenUsageEntry with cost_usd and subscription_id fields
- **AppState**: New `Arc<RwLock<HashMap<String, ModelPricing>>>` for cached pricing data, periodic refresh task
- **Redis**: New key patterns for cost counters (total, per-model, per-window), subscription activation (SETNX), subscription list cache
- **Admin API**: New routes for subscription_plans CRUD and key_subscriptions management
- **Frontend**: New Plans page + route, updated GroupDetailPage Keys tab, new store actions
- **Existing columns**: `group_keys.monthly_token_limit` and `group_keys.monthly_request_limit` dropped from Rust model, TypeScript interface, and database
