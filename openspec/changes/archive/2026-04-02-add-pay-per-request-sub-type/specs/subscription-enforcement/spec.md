## MODIFIED Requirements

### Requirement: Subscription selection priority
The proxy SHALL select the charging subscription using this priority: hourly_reset subscriptions first, then pay_per_request subscriptions, then fixed subscriptions. Within the same type, FIFO (oldest `created_at` first).

#### Scenario: Hourly_reset preferred over fixed
- **WHEN** a sub-key has both an active hourly_reset subscription with window budget and an active fixed subscription with budget
- **THEN** the system SHALL charge the hourly_reset subscription

#### Scenario: Pay_per_request preferred over fixed
- **WHEN** a sub-key has both an active pay_per_request subscription with budget and an active fixed subscription with budget
- **THEN** the system SHALL charge the pay_per_request subscription

#### Scenario: Hourly window full, fall through to fixed
- **WHEN** a sub-key has an hourly_reset subscription with current window exhausted and a fixed subscription with budget
- **THEN** the system SHALL skip the hourly_reset and charge the fixed subscription

#### Scenario: FIFO within same type
- **WHEN** a sub-key has two active fixed subscriptions, one created before the other
- **THEN** the system SHALL charge the older subscription first

### Requirement: Pay-per-request model blocking in subscription check
The proxy SHALL skip a `pay_per_request` subscription during the budget check if the requested model is not present as a key in `model_request_costs`. The subscription SHALL NOT be marked exhausted or expired; it is simply ineligible for this model.

#### Scenario: Model not in model_request_costs skips subscription
- **WHEN** a proxy request uses model "claude-opus-4-6" and the candidate subscription is `pay_per_request` with `model_request_costs: {"claude-sonnet-4-6": 0.10}`
- **THEN** the system SHALL skip this subscription and try the next one

#### Scenario: Model in model_request_costs proceeds to budget check
- **WHEN** a proxy request uses model "claude-sonnet-4-6" and the candidate subscription is `pay_per_request` with `model_request_costs: {"claude-sonnet-4-6": 0.10}` and remaining budget
- **THEN** the system SHALL allow the request and charge this subscription
