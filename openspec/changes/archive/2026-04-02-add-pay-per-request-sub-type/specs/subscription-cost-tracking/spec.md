## MODIFIED Requirements

### Requirement: Redis cost counters
The system SHALL maintain real-time cost counters in Redis for each active subscription. The windowing key pattern SHALL be used for any subscription where `reset_hours` is not null, regardless of `sub_type`.

#### Scenario: Fixed subscription counters
- **WHEN** a request is charged to a fixed subscription
- **THEN** the system SHALL INCRBYFLOAT `sub_cost:{sub_id}` by the request cost and INCRBYFLOAT `sub_cost:{sub_id}:m:{model}` by the request cost

#### Scenario: Windowed subscription counters (reset_hours is set)
- **WHEN** a request is charged to a subscription with `reset_hours` set (hourly_reset or pay_per_request with reset) with `window_idx = floor((now - activated_at) / (reset_hours * 3600))`
- **THEN** the system SHALL INCRBYFLOAT `sub_cost:{sub_id}:w:{window_idx}` and `sub_cost:{sub_id}:w:{window_idx}:m:{model}`, both with TTL equal to `reset_hours` in seconds

### Requirement: Redis counter rebuild on miss
The system SHALL rebuild Redis cost counters from the database when a Redis GET returns nil (cache miss). The windowing rebuild logic SHALL apply to any subscription where `reset_hours` is not null.

#### Scenario: Fixed subscription rebuild
- **WHEN** Redis key `sub_cost:{sub_id}` is missing
- **THEN** the system SHALL query `SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage_logs WHERE subscription_id = $1` and SET the result in Redis

#### Scenario: Per-model rebuild
- **WHEN** Redis key `sub_cost:{sub_id}:m:{model}` is missing
- **THEN** the system SHALL query `SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage_logs WHERE subscription_id = $1 AND model = $2` and SET the result in Redis

#### Scenario: Window rebuild for any subscription with reset_hours
- **WHEN** Redis key `sub_cost:{sub_id}:w:{window_idx}` is missing and the subscription has `reset_hours` set
- **THEN** the system SHALL query the sum of `cost_usd` from `token_usage_logs` where `subscription_id` matches and `created_at` falls within the current window boundaries, and SET the result with TTL
