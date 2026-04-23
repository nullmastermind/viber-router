## ADDED Requirements

### Requirement: Token usage aggregation by sub-key endpoint
The system SHALL provide a `GET /api/admin/token-usage/by-key` endpoint that returns token usage aggregated per `group_key_id` for a given group.

#### Scenario: Fetch usage by key with period shortcut
- **WHEN** an authenticated admin sends `GET /api/admin/token-usage/by-key?group_id=<uuid>&period=30d`
- **THEN** the system SHALL return JSON with `{ keys: [...] }` where each element contains: `group_key_id` (UUID or null), `key_name` (string or null), `created_at` (ISO timestamp or null), `total_input_tokens` (SUM), `total_output_tokens` (SUM), `total_cache_creation_tokens` (SUM), `total_cache_read_tokens` (SUM), `request_count` (COUNT), and `cost_usd` (calculated cost or null)

#### Scenario: Fetch usage by key with absolute date range
- **WHEN** an authenticated admin sends `GET /api/admin/token-usage/by-key?group_id=<uuid>&start=<iso>&end=<iso>`
- **THEN** the system SHALL return the same response shape with data filtered to the specified date range

#### Scenario: Period shortcuts supported
- **WHEN** the `period` parameter is one of `1h`, `6h`, `24h`, `7d`, `30d`
- **THEN** the system SHALL interpret it as a relative time window from now
- **AND** any unrecognized period value SHALL default to `24h`

#### Scenario: group_id is required
- **WHEN** the request omits `group_id`
- **THEN** the system SHALL return HTTP 400 with error message "group_id is required"

#### Scenario: NULL group_key_id aggregation
- **WHEN** token usage records exist where `group_key_id IS NULL` (master key or dynamic key usage)
- **THEN** those records SHALL be aggregated into a single row with `group_key_id: null` and `key_name: null`

#### Scenario: Cost calculation with model pricing
- **WHEN** a key's usage includes a model that has pricing configured in the `models` table
- **THEN** the `cost_usd` for that key SHALL be calculated using the same formula as the existing token-usage endpoint: `(input_tokens * input_1m_usd * rate_input + output_tokens * output_1m_usd * rate_output + cache_creation_tokens * cache_write_1m_usd * rate_cache_write + cache_read_tokens * cache_read_1m_usd * rate_cache_read) / 1,000,000`

#### Scenario: Cost calculation without model pricing
- **WHEN** a key's usage includes only models without pricing configured
- **THEN** `cost_usd` SHALL be null for that key

#### Scenario: Deleted key handling
- **WHEN** a `group_key_id` in token usage logs references a key that no longer exists in `group_keys`
- **THEN** the row SHALL still appear with `key_name: null` and `created_at: null` (LEFT JOIN behavior)

#### Scenario: Result ordering
- **WHEN** the endpoint returns results
- **THEN** rows SHALL be ordered by `request_count` descending (highest usage first)
