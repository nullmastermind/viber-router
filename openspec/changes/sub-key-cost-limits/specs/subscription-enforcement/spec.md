## ADDED Requirements

### Requirement: Pre-request subscription budget check
The proxy SHALL check subscription budgets before forwarding a request when the sub-key has any subscriptions. The check SHALL use Redis counters for performance.

#### Scenario: Sub-key with no subscriptions
- **WHEN** a proxy request uses a sub-key that has never had any subscriptions
- **THEN** the system SHALL allow the request without any budget check (unlimited)

#### Scenario: Sub-key with active subscription and available budget
- **WHEN** a proxy request uses a sub-key with an active subscription that has remaining budget
- **THEN** the system SHALL allow the request

#### Scenario: All subscriptions exhausted or expired
- **WHEN** a proxy request uses a sub-key where all subscriptions are in terminal states (exhausted, expired, cancelled)
- **THEN** the system SHALL return 429 with Anthropic-format error: `{"type":"error","error":{"type":"rate_limit_error","message":"Subscription limit exceeded"}}`

### Requirement: Subscription selection priority
The proxy SHALL select the charging subscription using this priority: hourly_reset subscriptions first, then fixed subscriptions. Within the same type, FIFO (oldest `created_at` first).

#### Scenario: Hourly_reset preferred over fixed
- **WHEN** a sub-key has both an active hourly_reset subscription with window budget and an active fixed subscription with budget
- **THEN** the system SHALL charge the hourly_reset subscription

#### Scenario: Hourly window full, fall through to fixed
- **WHEN** a sub-key has an hourly_reset subscription with current window exhausted and a fixed subscription with budget
- **THEN** the system SHALL skip the hourly_reset and charge the fixed subscription

#### Scenario: FIFO within same type
- **WHEN** a sub-key has two active fixed subscriptions, one created before the other
- **THEN** the system SHALL charge the older subscription first

### Requirement: Per-model budget enforcement
The proxy SHALL enforce per-model cost limits defined in a subscription's `model_limits` JSONB. Model names SHALL match exactly against the request model (from the `models` table).

#### Scenario: Model limit not exceeded
- **WHEN** a request for model "claude-opus-4-6" hits a subscription with `model_limits: {"claude-opus-4-6": 30.0}` and current opus cost is $25
- **THEN** the system SHALL allow the request and charge this subscription

#### Scenario: Model limit exceeded, skip subscription
- **WHEN** a request for model "claude-opus-4-6" hits a subscription with `model_limits: {"claude-opus-4-6": 30.0}` and current opus cost is $30
- **THEN** the system SHALL skip this subscription and try the next one (subscription is NOT marked EXHAUSTED)

#### Scenario: No model limit configured
- **WHEN** a request for model "claude-sonnet-4-6" hits a subscription with `model_limits: {"claude-opus-4-6": 30.0}` (no sonnet limit)
- **THEN** the system SHALL only check the total budget, not per-model

#### Scenario: Per-model limits reset with hourly window
- **WHEN** an hourly_reset subscription's window resets
- **THEN** per-model cost counters for that window SHALL also reset

### Requirement: Expiration enforcement
The proxy SHALL check subscription expiration during the budget check. Expired subscriptions SHALL be marked with `status: "expired"` in the database.

#### Scenario: Subscription expired
- **WHEN** the current time exceeds a subscription's `expires_at`
- **THEN** the system SHALL mark it as `expired` and skip to the next subscription

### Requirement: Exhaustion detection
The proxy SHALL mark a subscription as `exhausted` when its total cost reaches `cost_limit_usd`. For hourly_reset subscriptions, exhaustion means the subscription's total lifetime cost across all windows has reached the limit — individual window exhaustion is temporary and resets.

#### Scenario: Fixed subscription exhausted
- **WHEN** a fixed subscription's total cost reaches `cost_limit_usd`
- **THEN** the system SHALL mark it as `exhausted` permanently

#### Scenario: Hourly window temporarily full
- **WHEN** an hourly_reset subscription's current window cost reaches `cost_limit_usd`
- **THEN** the system SHALL skip it for this request but NOT mark it as `exhausted` (it resets next window)
