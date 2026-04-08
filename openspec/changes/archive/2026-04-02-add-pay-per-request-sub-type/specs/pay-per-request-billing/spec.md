## ADDED Requirements

### Requirement: Pay-per-request flat cost billing
When a subscription has `sub_type = "pay_per_request"`, the proxy SHALL calculate the request cost as the flat value from `model_request_costs[model]` instead of the token-based formula. The `calculate_cost()` function SHALL NOT be called for this subscription type.

#### Scenario: Flat cost applied after response
- **WHEN** a proxy request completes and the active subscription is `pay_per_request` with `model_request_costs: {"claude-sonnet-4-6": 0.10}`
- **THEN** the cost charged SHALL be 0.10 regardless of token count

#### Scenario: Flat cost applied in streaming path
- **WHEN** a streaming proxy request completes and the active subscription is `pay_per_request`
- **THEN** the flat cost from `model_request_costs[model]` SHALL be charged, not the token-based cost

### Requirement: Block request for unlisted model
When a subscription has `sub_type = "pay_per_request"` and the requested model is not a key in `model_request_costs`, the proxy SHALL skip this subscription (treat it as if it has no budget for this model). If no other subscription covers the request, the request SHALL be blocked with 429.

#### Scenario: Model not in model_request_costs
- **WHEN** a proxy request uses model "claude-opus-4-6" and the active `pay_per_request` subscription has `model_request_costs: {"claude-sonnet-4-6": 0.10}` (no opus entry)
- **THEN** the system SHALL skip this subscription for this request

#### Scenario: Only pay_per_request subscription, model unlisted
- **WHEN** a sub-key has only one active subscription of type `pay_per_request` and the requested model is not in `model_request_costs`
- **THEN** the system SHALL return 429 with Anthropic-format error

### Requirement: Pay-per-request with optional reset window
A `pay_per_request` subscription SHALL support an optional `reset_hours` field. When `reset_hours` is set, the budget resets every N hours (same windowing logic as `hourly_reset`). When `reset_hours` is null, the budget is a one-time lifetime limit.

#### Scenario: Pay-per-request with reset window
- **WHEN** a `pay_per_request` subscription has `reset_hours: 24` and the current window budget is exhausted
- **THEN** the system SHALL skip this subscription for the current window but NOT mark it as exhausted (it resets next window)

#### Scenario: Pay-per-request without reset window
- **WHEN** a `pay_per_request` subscription has `reset_hours: null` and total cost reaches `cost_limit_usd`
- **THEN** the system SHALL mark it as `exhausted` permanently
