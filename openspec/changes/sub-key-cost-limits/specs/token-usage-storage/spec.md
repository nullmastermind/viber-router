## MODIFIED Requirements

### Requirement: Token usage log table
The system SHALL persist token usage records in a partitioned `token_usage_logs` PostgreSQL table with columns: `id` (UUID), `created_at` (TIMESTAMPTZ), `group_id` (UUID), `server_id` (UUID), `model` (TEXT, nullable), `input_tokens` (INTEGER), `output_tokens` (INTEGER), `cache_creation_tokens` (INTEGER, nullable), `cache_read_tokens` (INTEGER, nullable), `is_dynamic_key` (BOOLEAN), `key_hash` (TEXT, nullable), `group_key_id` (UUID, nullable), `cost_usd` (FLOAT8, nullable), `subscription_id` (UUID, nullable), partitioned by range on `created_at`.

#### Scenario: Usage record persisted
- **WHEN** the proxy extracts token usage from a successful `/v1/messages` response
- **THEN** the system SHALL insert a record into `token_usage_logs` with the group_id, server_id, model, input_tokens, output_tokens, cache token counts, is_dynamic_key flag, key_hash, group_key_id, cost_usd, and subscription_id

#### Scenario: Dynamic key usage
- **WHEN** the request used a dynamic key (via `-rsv-` syntax) for the winning server
- **THEN** the record SHALL have `is_dynamic_key: true` and `key_hash` set to the first 16 hex characters of the SHA-256 hash of the dynamic key

#### Scenario: Server default key usage
- **WHEN** the request used the server's default API key (no dynamic key for that server)
- **THEN** the record SHALL have `is_dynamic_key: false` and `key_hash` set to the first 16 hex characters of the SHA-256 hash of the server's default key

#### Scenario: Cost recorded
- **WHEN** model pricing is available for the request model
- **THEN** the record SHALL have `cost_usd` set to the calculated cost using model pricing × server rates

#### Scenario: Cost not available
- **WHEN** model pricing is not found for the request model
- **THEN** the record SHALL have `cost_usd: NULL`

#### Scenario: Subscription charged
- **WHEN** the request was charged to a subscription
- **THEN** the record SHALL have `subscription_id` set to the charged subscription's UUID

#### Scenario: No subscription charged
- **WHEN** the request was not charged to any subscription (unlimited key or pricing unavailable)
- **THEN** the record SHALL have `subscription_id: NULL`
