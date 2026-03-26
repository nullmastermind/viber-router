## ADDED Requirements

### Requirement: Redis cost counters
The system SHALL maintain real-time cost counters in Redis for each active subscription.

#### Scenario: Fixed subscription counters
- **WHEN** a request is charged to a fixed subscription
- **THEN** the system SHALL INCRBYFLOAT `sub_cost:{sub_id}` by the request cost and INCRBYFLOAT `sub_cost:{sub_id}:m:{model}` by the request cost

#### Scenario: Hourly_reset subscription counters
- **WHEN** a request is charged to an hourly_reset subscription with `window_idx = floor((now - activated_at) / (reset_hours * 3600))`
- **THEN** the system SHALL INCRBYFLOAT `sub_cost:{sub_id}:w:{window_idx}` and `sub_cost:{sub_id}:w:{window_idx}:m:{model}`, both with TTL equal to `reset_hours` in seconds

### Requirement: Redis counter rebuild on miss
The system SHALL rebuild Redis cost counters from the database when a Redis GET returns nil (cache miss).

#### Scenario: Fixed subscription rebuild
- **WHEN** Redis key `sub_cost:{sub_id}` is missing
- **THEN** the system SHALL query `SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage_logs WHERE subscription_id = $1` and SET the result in Redis

#### Scenario: Per-model rebuild
- **WHEN** Redis key `sub_cost:{sub_id}:m:{model}` is missing
- **THEN** the system SHALL query `SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage_logs WHERE subscription_id = $1 AND model = $2` and SET the result in Redis

#### Scenario: Hourly window rebuild
- **WHEN** Redis key `sub_cost:{sub_id}:w:{window_idx}` is missing
- **THEN** the system SHALL query the sum of `cost_usd` from `token_usage_logs` where `subscription_id` matches and `created_at` falls within the current window boundaries, and SET the result with TTL

### Requirement: Model pricing cache
The system SHALL cache model pricing data in `AppState` as `Arc<RwLock<HashMap<String, ModelPricing>>>` where `ModelPricing` contains `input_1m_usd`, `output_1m_usd`, `cache_write_1m_usd`, `cache_read_1m_usd`. The cache SHALL refresh every 60 seconds via a background task.

#### Scenario: Pricing cache refresh
- **WHEN** 60 seconds have elapsed since the last refresh
- **THEN** the system SHALL query all rows from the `models` table and update the pricing cache

#### Scenario: Pricing cache used for cost calculation
- **WHEN** the proxy calculates cost for a request with model "claude-opus-4-6"
- **THEN** the system SHALL look up pricing from the cache using the model name as key

### Requirement: Cost calculation formula
The system SHALL calculate cost as: `cost_usd = (input_tokens × input_1m_usd × rate_input + output_tokens × output_1m_usd × rate_output + cache_creation_tokens × cache_write_1m_usd × rate_cache_write + cache_read_tokens × cache_read_1m_usd × rate_cache_read) / 1,000,000`. Server rate multipliers SHALL be obtained from the `group_servers` record for the group and server that handled the request.

#### Scenario: Cost with all token types
- **WHEN** a request produces 1000 input tokens, 500 output tokens, 200 cache creation tokens, 100 cache read tokens for model with pricing ($3/1M input, $15/1M output, $3.75/1M cache write, $0.30/1M cache read) and server rates all 1.0
- **THEN** cost_usd SHALL be `(1000×3 + 500×15 + 200×3.75 + 100×0.30) / 1,000,000 = 0.01128`

#### Scenario: Model pricing not found
- **WHEN** the model name is not found in the pricing cache
- **THEN** cost_usd SHALL be NULL and the subscription SHALL NOT be charged

### Requirement: Subscription list cache
The system SHALL cache the list of active subscriptions per sub-key in Redis at key `key_subs:{group_key_id}`. The cache SHALL be invalidated when subscriptions are added, cancelled, or status changes.

#### Scenario: Cache hit
- **WHEN** the proxy checks subscriptions for a sub-key and `key_subs:{group_key_id}` exists in Redis
- **THEN** the system SHALL use the cached subscription list

#### Scenario: Cache miss
- **WHEN** `key_subs:{group_key_id}` is missing from Redis
- **THEN** the system SHALL query `key_subscriptions` from the database, cache the result, and use it

#### Scenario: Cache invalidation on subscription change
- **WHEN** a subscription is added or cancelled for a sub-key
- **THEN** the system SHALL DELETE `key_subs:{group_key_id}` from Redis

### Requirement: Activation via Redis SETNX
The system SHALL use Redis SETNX on key `sub_activated:{sub_id}` to handle concurrent first-request activation. The value SHALL be the activation timestamp. After SETNX, the system SHALL also update the database record with `activated_at` and `expires_at`.

#### Scenario: First activation
- **WHEN** a subscription with `activated_at: NULL` receives its first charge and SETNX succeeds
- **THEN** the system SHALL set `activated_at` and `expires_at` in both Redis and the database

#### Scenario: Concurrent activation
- **WHEN** SETNX fails (key already exists)
- **THEN** the system SHALL GET the stored activation timestamp and use it
