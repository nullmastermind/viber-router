## Purpose
TBD

## Requirements
### Requirement: Cost calculation formula
The system SHALL calculate cost for each token usage row using the formula: `cost = tokens × model_price_per_1m × server_rate / 1_000_000` for each of the 4 token types (input, output, cache write, cache read). The total cost for a row is the sum of all 4 type costs.

#### Scenario: Full pricing and default rates
- **WHEN** a usage row has 1000 input tokens, the model has `input_1m_usd = 3.0`, and the server rate is NULL (default 1.0)
- **THEN** the input cost SHALL be `1000 × 3.0 × 1.0 / 1_000_000 = $0.003`

#### Scenario: Custom server rate
- **WHEN** a usage row has 1000 input tokens, the model has `input_1m_usd = 3.0`, and the server has `rate_input = 1.5`
- **THEN** the input cost SHALL be `1000 × 3.0 × 1.5 / 1_000_000 = $0.0045`

#### Scenario: Model has no pricing
- **WHEN** the model name in `token_usage_logs` does not match any model in the `models` table, or the matched model has NULL pricing
- **THEN** the cost for that row SHALL be NULL

#### Scenario: Zero tokens
- **WHEN** a token count is 0 (e.g., cache_creation_tokens = 0)
- **THEN** the cost contribution for that type SHALL be $0.00

#### Scenario: Server removed from group
- **WHEN** a usage row references a server_id that is no longer in the group's `group_servers`
- **THEN** the system SHALL use rate 1.0 (LEFT JOIN returns NULL, COALESCE to 1.0)

### Requirement: Model matching strategy
The system SHALL match `token_usage_logs.model` against `models.name` using exact string equality. No fuzzy matching or alias resolution.

#### Scenario: Exact match
- **WHEN** `token_usage_logs.model` is "claude-sonnet-4-20250514" and a model with name "claude-sonnet-4-20250514" exists
- **THEN** the system SHALL use that model's pricing for cost calculation

#### Scenario: No match
- **WHEN** `token_usage_logs.model` is "my-custom-alias" and no model with that name exists
- **THEN** the cost SHALL be NULL (displayed as "—")
