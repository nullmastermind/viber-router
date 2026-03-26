## Context

Viber Router proxies API requests through groups, each with a master key and optional sub-keys. Sub-keys currently have no cost enforcement — once active, they can consume unlimited resources. The system already tracks token usage per sub-key (`group_key_id` in `token_usage_logs`) and has model pricing data (`models` table with per-token USD rates) and server rate multipliers (`group_servers.rate_input/output/cache_write/cache_read`). Cost is currently computed only at query time in the stats API — not stored or enforced.

The subscription system introduces a plan-based cost limiting model where administrators define reusable plan templates globally, then assign them to sub-keys. Each assignment snapshots the plan configuration, creating an independent subscription instance with its own lifecycle and cost tracking.

## Goals / Non-Goals

**Goals:**
- Enable cost-based limits on sub-keys via subscription plans
- Support two subscription types: fixed budget (lifetime) and hourly-reset (windowed)
- Support per-model cost limits within subscriptions
- Real-time enforcement in the proxy with minimal latency impact
- Global plan management for efficient multi-key administration
- API-driven subscription assignment for 3rd party integration

**Non-Goals:**
- Token-based or request-count-based limits (replaced by cost-based approach)
- Notification/alerting when approaching limits (future work)
- Subscription billing or payment integration
- Cost limits on master keys (only sub-keys)
- Rollover of unused budget between subscriptions

## Decisions

### 1. Subscription plan catalog (template + snapshot pattern)

Plans are defined globally in `subscription_plans` table. When assigned to a sub-key, all plan values are copied into `key_subscriptions`. This means plan edits don't affect existing subscriptions — like an order capturing product price at purchase time.

**Alternative**: Reference-only (FK to plan, no snapshot). Rejected because changing a plan would silently alter active subscriptions' terms.

### 2. Cost formula includes server rate multipliers

`cost_usd = (tokens × model_price × server_rate) / 1,000,000`

The subscription limit represents cost after markup/discount, matching what the admin sees in the usage stats UI. This keeps the mental model consistent — the cost shown in usage reports is the same cost counted against the subscription.

**Alternative**: Raw model pricing without server rates. Rejected because it would create a disconnect between reported costs and subscription consumption.

### 3. Cost calculated in proxy, not in usage buffer

The proxy calculates cost immediately after a successful response and updates Redis counters in the same flow. This ensures the pre-request budget check uses fresh data. Model pricing is cached in `AppState` via `Arc<RwLock<HashMap<String, ModelPricing>>>`, refreshed every 60 seconds.

For streaming responses, the `UsageTrackingStream` already holds `state: AppState` and performs the Redis update when the stream completes.

**Alternative**: Calculate in `flush_task` (usage buffer). Rejected because the buffer flushes every 5 seconds, creating a stale window where the budget check could allow requests that should be blocked.

### 4. Redis for real-time cost tracking with DB rebuild fallback

Redis keys track running cost totals. On Redis miss (restart/flush), the system rebuilds from `SUM(cost_usd) FROM token_usage_logs WHERE subscription_id = X`. This provides fast enforcement with durability.

Key patterns:
- Fixed: `sub_cost:{sub_id}` (total), `sub_cost:{sub_id}:m:{model}` (per-model)
- Hourly: `sub_cost:{sub_id}:w:{window_idx}` (window total), `sub_cost:{sub_id}:w:{window_idx}:m:{model}` (window per-model), with TTL = reset_hours
- Activation: `sub_activated:{sub_id}` via SETNX for race-safe first-request activation
- Subscription list: `key_subs:{group_key_id}` caching active subscriptions

Window index calculation: `floor((now - activated_at).as_secs() / (reset_hours * 3600))`

### 5. Multi-subscription charging priority

All subscriptions for a key are active simultaneously. Charging priority: hourly_reset first, then fixed, FIFO within same type. When an hourly window is full, the system falls through to fixed subscriptions — hourly_reset is a preferred charging target, not a hard rate limiter.

Per-model limit exceeded → skip that subscription for this model (not marked EXHAUSTED). Total budget exhausted → mark EXHAUSTED permanently.

### 6. Lazy activation

`activated_at` is set on the first request that charges a subscription, not at creation time. `expires_at` is computed as `activated_at + duration_days`. This means the subscription clock starts ticking on first use. Race condition handled via Redis SETNX — first request wins, subsequent requests read the stored value.

### 7. Drop unused monthly limit columns

`group_keys.monthly_token_limit` and `group_keys.monthly_request_limit` are removed. These were placeholder columns from the sub-keys feature, never enforced. The subscription system fully replaces them. Blast radius is contained: only `GroupKey` Rust struct, TypeScript interface, and the migration.

## Risks / Trade-offs

**[Overshoot by 1 request]** → Accepted. Cost is only known after the response (need token counts). Pre-check uses accumulated cost before the current request. Maximum overshoot = cost of one request. Acceptable for this use case.

**[Redis counter drift]** → Mitigated by DB rebuild on cache miss. If Redis loses data, counters are reconstructed from `token_usage_logs`. Small window of potential overshoot during rebuild.

**[Model pricing cache staleness]** → 60-second refresh interval means pricing changes take up to 60s to propagate. Acceptable — pricing changes are infrequent admin operations.

**[Streaming response cost delay]** → For streaming responses, Redis is updated only when the stream completes (could be minutes for long responses). During streaming, the budget check for concurrent requests uses pre-stream totals. Mitigated by the accepted overshoot policy.

**[cost_usd NULL for unknown models]** → If model pricing is not found in cache, the request is allowed with `cost_usd = NULL` and no subscription charge. This prevents blocking legitimate requests due to missing pricing config, but means unpriced models don't count against limits.
