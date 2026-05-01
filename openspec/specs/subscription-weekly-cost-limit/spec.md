# subscription-weekly-cost-limit Specification

## Purpose
TBD - created by archiving change add-weekly-cost-limit-timezone. Update Purpose after archive.
## Requirements
### Requirement: Weekly cost limit storage
The system SHALL store an optional `weekly_cost_limit_usd` value on subscription plans and assigned key subscriptions. A NULL value SHALL mean unlimited weekly cost and SHALL disable weekly limit enforcement for that subscription.

#### Scenario: Weekly limit columns exist
- **WHEN** the weekly limit migration runs
- **THEN** `subscription_plans.weekly_cost_limit_usd` and `key_subscriptions.weekly_cost_limit_usd` SHALL exist as nullable FLOAT8 columns

#### Scenario: Unlimited weekly limit
- **WHEN** a non-bonus subscription has `weekly_cost_limit_usd = NULL`
- **THEN** the system SHALL NOT perform a weekly cost limit check for that subscription

### Requirement: Weekly cost window
The system SHALL calculate weekly cost windows as calendar weeks in the configured global timezone, starting Monday 00:00 inclusive and ending the next Monday 00:00 exclusive. The default timezone SHALL be `Asia/Ho_Chi_Minh`.

#### Scenario: Current week key uses Monday epoch
- **WHEN** the system tracks weekly cost for a subscription
- **THEN** it SHALL use Redis key `sub_weekly_cost:{sub_id}:w:{monday_epoch}` where `monday_epoch` is the UTC epoch for Monday 00:00 in the configured timezone

#### Scenario: Weekly window ends at next Monday
- **WHEN** the system computes the weekly counter TTL
- **THEN** the TTL SHALL be the number of seconds until the next Monday 00:00 in the configured timezone

### Requirement: Weekly cost counter lookup
The system SHALL read weekly cost from Redis when available and rebuild the value from `token_usage_logs` on cache miss. The rebuild query SHALL sum `cost_usd` for the subscription where `created_at` is greater than or equal to the current week start UTC and less than the next week start UTC.

#### Scenario: Redis weekly cost hit
- **WHEN** the weekly Redis counter exists for a subscription and current week
- **THEN** the system SHALL use that counter value without querying token usage logs

#### Scenario: Redis weekly cost miss
- **WHEN** the weekly Redis counter is missing for a subscription and current week
- **THEN** the system SHALL rebuild the cost from `token_usage_logs`, store it in Redis under the current weekly key, and set TTL until next Monday 00:00 in the configured timezone

### Requirement: Weekly cost increment
The system SHALL increment the weekly Redis counter when updating subscription cost counters after a billable request, but only for non-bonus subscriptions with a non-NULL `weekly_cost_limit_usd`.

#### Scenario: Increment weekly counter
- **WHEN** a billable request records `cost_usd = 1.25` for a non-bonus subscription with a weekly limit
- **THEN** `update_cost_counters()` SHALL increment that subscription's current weekly Redis counter by 1.25 and ensure it expires at the next Monday 00:00 in the configured timezone

#### Scenario: Bonus subscription not tracked
- **WHEN** a bonus subscription handles usage
- **THEN** the system SHALL NOT create or increment a weekly cost counter for that bonus subscription

### Requirement: Weekly limit public reporting
The system SHALL expose weekly usage information for non-bonus subscriptions in public usage responses using `weekly_cost_used`, `weekly_cost_limit_usd`, and `weekly_reset_at`.

#### Scenario: Limited subscription public fields
- **WHEN** public usage is requested for a non-bonus subscription with `weekly_cost_limit_usd = 25.0` and current weekly usage is 10.5
- **THEN** the subscription object SHALL include `weekly_cost_used: 10.5`, `weekly_cost_limit_usd: 25.0`, and `weekly_reset_at` set to the next Monday 00:00 boundary as an ISO 8601 timestamp

#### Scenario: Unlimited subscription public fields
- **WHEN** public usage is requested for a non-bonus subscription with `weekly_cost_limit_usd = NULL`
- **THEN** the subscription object SHALL include `weekly_cost_limit_usd: null` and SHALL NOT imply that a weekly limit applies

