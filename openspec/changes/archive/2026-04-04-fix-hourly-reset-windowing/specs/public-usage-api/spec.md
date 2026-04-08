## MODIFIED Requirements

### Requirement: Subscription data includes cost_used and window reset time
The system SHALL return non-cancelled subscriptions (`active`, `exhausted`, `expired`) for the sub-key, each enriched with `cost_used` (current cost from Redis/DB) and `window_reset_at` (for subscriptions with `reset_hours` set, when an active window exists). Subscriptions with status `cancelled` are excluded.

#### Scenario: Active fixed subscription
- **WHEN** a sub-key has an active fixed subscription with $2.50 used of $10.00 limit
- **THEN** the subscription entry includes `cost_used: 2.50`, `cost_limit_usd: 10.00`, `window_reset_at: null`

#### Scenario: Active hourly_reset subscription with active window
- **WHEN** a sub-key has an active hourly_reset subscription with `reset_hours: 4` and `sub_window_start:{sub_id}` exists in Redis with epoch `ws`
- **THEN** the subscription entry includes `cost_used` for the current window and `window_reset_at` equal to `DateTime::from_timestamp(ws) + 4 hours` formatted as ISO 8601

#### Scenario: Active hourly_reset subscription with no active window
- **WHEN** a sub-key has an active hourly_reset subscription with `reset_hours: 4` and `sub_window_start:{sub_id}` does not exist in Redis
- **THEN** the subscription entry includes `cost_used: 0.0` and `window_reset_at: null`

#### Scenario: Expired/exhausted subscriptions
- **WHEN** a sub-key has subscriptions with status `expired` or `exhausted`
- **THEN** these subscriptions are included in the response with their actual status and `cost_used: 0.0`

#### Scenario: Cancelled subscriptions are excluded
- **WHEN** a sub-key has subscriptions with status `cancelled`
- **THEN** these subscriptions are NOT included in the response
