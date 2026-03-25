## ADDED Requirements

### Requirement: Token usage log table
The system SHALL persist token usage records in a partitioned `token_usage_logs` PostgreSQL table with columns: `id` (UUID), `created_at` (TIMESTAMPTZ), `group_id` (UUID), `server_id` (UUID), `model` (TEXT, nullable), `input_tokens` (INTEGER), `output_tokens` (INTEGER), `cache_creation_tokens` (INTEGER, nullable), `cache_read_tokens` (INTEGER, nullable), `is_dynamic_key` (BOOLEAN), `key_hash` (TEXT, nullable), partitioned by range on `created_at`.

#### Scenario: Usage record persisted
- **WHEN** the proxy extracts token usage from a successful `/v1/messages` response
- **THEN** the system SHALL insert a record into `token_usage_logs` with the group_id, server_id, model, input_tokens, output_tokens, cache token counts, is_dynamic_key flag, and key_hash

#### Scenario: Dynamic key usage
- **WHEN** the request used a dynamic key (via `-rsv-` syntax) for the winning server
- **THEN** the record SHALL have `is_dynamic_key: true` and `key_hash` set to the first 16 hex characters of the SHA-256 hash of the dynamic key

#### Scenario: Server default key usage
- **WHEN** the request used the server's default API key (no dynamic key for that server)
- **THEN** the record SHALL have `is_dynamic_key: false` and `key_hash` set to the first 16 hex characters of the SHA-256 hash of the server's default key

### Requirement: Buffered batch inserts
The system SHALL buffer token usage entries in memory and flush them to PostgreSQL in batches, using the same mpsc channel pattern as `proxy_logs` and `ttft_logs`.

#### Scenario: Batch flush on interval
- **WHEN** the flush interval (5 seconds) elapses and the buffer is non-empty
- **THEN** the system SHALL INSERT all buffered entries using UNNEST batch insert and clear the buffer

#### Scenario: Batch flush on capacity
- **WHEN** the buffer reaches 100 entries
- **THEN** the system SHALL INSERT all buffered entries immediately

#### Scenario: Buffer full
- **WHEN** the mpsc channel is full (capacity 10,000)
- **THEN** the system SHALL drop the entry and log a warning via `tracing::warn`

#### Scenario: Graceful shutdown
- **WHEN** the application receives a shutdown signal
- **THEN** the system SHALL flush any remaining buffered entries before exiting

### Requirement: Partition maintenance
The system SHALL create and maintain monthly partitions for `token_usage_logs` following the same pattern as `proxy_logs` and `ttft_logs`.

#### Scenario: Startup partition creation
- **WHEN** the application starts
- **THEN** the system SHALL ensure partitions exist for the current and next month for `token_usage_logs`

#### Scenario: Daily partition maintenance
- **WHEN** the daily maintenance task runs
- **THEN** the system SHALL ensure partitions and drop expired partitions for `token_usage_logs` using the configured retention period

### Requirement: Table indexes
The system SHALL create indexes on `token_usage_logs` to support efficient aggregation queries.

#### Scenario: Indexes created
- **WHEN** the migration runs
- **THEN** the system SHALL create indexes on `(group_id, created_at)` and `(key_hash, created_at)`
