## Purpose
TBD

## Requirements
### Requirement: Uptime checks table
The system SHALL store uptime check records in an `uptime_checks` table partitioned by RANGE on `created_at` with columns: `id` (UUID PK), `created_at` (TIMESTAMPTZ), `group_id` (UUID NOT NULL), `server_id` (UUID NOT NULL), `status_code` (SMALLINT NOT NULL), `latency_ms` (INTEGER NOT NULL), `request_id` (UUID NOT NULL). Indexes SHALL exist on `(group_id, server_id, created_at)` and `(group_id, created_at)`.

#### Scenario: Table creation
- **WHEN** the database migration runs
- **THEN** the `uptime_checks` parent table SHALL be created as PARTITION BY RANGE (created_at) with the specified columns and indexes

#### Scenario: Data routed to correct partition
- **WHEN** an uptime check entry with created_at in March 2026 is inserted
- **THEN** the entry SHALL be stored in the `uptime_checks_2026_03` partition

### Requirement: Async uptime buffer
The system SHALL use a bounded mpsc channel (capacity 10,000) to buffer uptime check entries. A background flush task SHALL batch-insert entries to PostgreSQL every 5 seconds or when 100 entries accumulate, whichever comes first.

#### Scenario: Normal operation
- **WHEN** the proxy emits an uptime check entry
- **THEN** the entry SHALL be sent via `try_send` to the mpsc channel without blocking the proxy handler

#### Scenario: Buffer full
- **WHEN** the uptime mpsc channel is full (10,000 entries) and a new entry is produced
- **THEN** the new entry SHALL be dropped and a `tracing::warn` SHALL be emitted

#### Scenario: Database flush failure
- **WHEN** a batch INSERT to PostgreSQL fails
- **THEN** the batch SHALL be dropped, a `tracing::warn` SHALL be emitted, and the flush task SHALL continue processing new entries

### Requirement: Partition management for uptime_checks
The system SHALL ensure current and next month partitions exist for `uptime_checks` on startup and during daily maintenance. Expired partitions SHALL be dropped based on `LOG_RETENTION_DAYS`.

#### Scenario: Startup partition creation
- **WHEN** the application starts
- **THEN** the system SHALL ensure partitions exist for the current month and next month for `uptime_checks`

#### Scenario: Daily maintenance
- **WHEN** the daily partition maintenance job runs
- **THEN** the system SHALL create the next month's partition if missing and drop partitions older than `LOG_RETENTION_DAYS` for `uptime_checks`

### Requirement: Proxy emits uptime check for each server attempt
The proxy handler SHALL generate a `request_id` UUID at the start of each request. For each server attempt in the failover chain (including the count-tokens default server if attempted), the proxy SHALL emit an uptime check entry with: group_id, server_id, status_code (0 for connection errors), latency_ms, request_id, and created_at.

#### Scenario: Single server success
- **WHEN** a proxy request tries server A and receives HTTP 200
- **THEN** the system SHALL emit exactly 1 uptime check entry with server_id=A, status_code=200

#### Scenario: Failover chain
- **WHEN** a proxy request tries server A (429), then server B (200)
- **THEN** the system SHALL emit 2 uptime check entries: one with server_id=A, status_code=429 and one with server_id=B, status_code=200, both sharing the same request_id

#### Scenario: Connection error
- **WHEN** a proxy request tries a server and the connection fails
- **THEN** the system SHALL emit an uptime check entry with status_code=0

#### Scenario: All servers exhausted
- **WHEN** a proxy request tries all servers and none succeed
- **THEN** the system SHALL emit one uptime check entry per server attempted, all sharing the same request_id
