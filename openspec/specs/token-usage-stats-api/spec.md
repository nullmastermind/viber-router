## Purpose
TBD

## Requirements
### Requirement: Token usage statistics API endpoint
The system SHALL provide a GET endpoint at `/api/admin/token-usage` that returns aggregated token usage statistics per server for a given group within a time range. The endpoint SHALL require admin authentication. The response SHALL include cost fields calculated by joining model pricing and server rate data.

#### Scenario: Fetch token usage with absolute date range
- **WHEN** an authenticated admin sends `GET /api/admin/token-usage?group_id=<uuid>&start=<iso>&end=<iso>`
- **THEN** the system SHALL return JSON with per-server/model aggregated statistics including: server_id, server_name, model, total_input_tokens (SUM), total_output_tokens (SUM), total_cache_creation_tokens (SUM), total_cache_read_tokens (SUM), request_count (COUNT), and cost_usd (calculated total cost for the row, or null if model has no pricing)

#### Scenario: Fetch token usage with relative period
- **WHEN** an authenticated admin sends `GET /api/admin/token-usage?group_id=<uuid>&period=24h`
- **THEN** the system SHALL return aggregated statistics for the last 24 hours including cost fields

#### Scenario: Filter by dynamic key
- **WHEN** the request includes `is_dynamic_key=true`
- **THEN** the system SHALL only include records where `is_dynamic_key` is true

#### Scenario: Filter by key hash
- **WHEN** the request includes `key_hash=<hash>`
- **THEN** the system SHALL only include records matching that key_hash

#### Scenario: No usage data available
- **WHEN** no token usage records exist for the given group and time range
- **THEN** the system SHALL return an empty `servers` array

#### Scenario: Missing group_id parameter
- **WHEN** the request omits the `group_id` query parameter
- **THEN** the system SHALL return HTTP 400 with an error message

#### Scenario: Unauthenticated request
- **WHEN** a request to `/api/admin/token-usage` has no valid admin token
- **THEN** the system SHALL return HTTP 401 (handled by existing admin auth middleware)

#### Scenario: Model with no pricing configured
- **WHEN** a usage row's model does not match any model in the `models` table or the matched model has NULL pricing
- **THEN** the `cost_usd` field for that row SHALL be null

#### Scenario: Cost calculation with server rates
- **WHEN** a usage row's server has non-default rate multipliers in `group_servers`
- **THEN** the cost calculation SHALL apply the rate multipliers to the respective token type costs
