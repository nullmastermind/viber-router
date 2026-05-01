## Purpose
TBD

## Requirements
### Requirement: Monthly range partitioning for proxy_logs
The proxy_logs table SHALL be created as a PostgreSQL range-partitioned table on the created_at column with monthly partitions. Partition tables SHALL be named proxy_logs_YYYY_MM (e.g., proxy_logs_2026_03).

#### Scenario: Table creation
- **WHEN** the database migration runs
- **THEN** the proxy_logs parent table SHALL be created as PARTITION BY RANGE (created_at) with appropriate indexes on created_at, group_id, status_code, and group_api_key

#### Scenario: Data routed to correct partition
- **WHEN** a log entry with created_at in March 2026 is inserted
- **THEN** the entry SHALL be stored in the proxy_logs_2026_03 partition

### Requirement: Automatic partition creation on startup
The application SHALL ensure that partitions exist for the current month and the next month when it starts. Partitions SHALL be created using CREATE TABLE IF NOT EXISTS to handle concurrent startup safely.

#### Scenario: Fresh start with no partitions
- **WHEN** the application starts and no monthly partitions exist
- **THEN** the system SHALL create partitions for the current month and next month

#### Scenario: Partitions already exist
- **WHEN** the application starts and partitions for current and next month already exist
- **THEN** the system SHALL skip creation without error (IF NOT EXISTS)

### Requirement: Daily partition maintenance job
The application SHALL run a background job once per day that creates the next month's partition (if not exists) and drops partitions whose date range is entirely older than LOG_RETENTION_DAYS.

#### Scenario: Daily job creates upcoming partition
- **WHEN** the daily maintenance job runs on March 24, 2026
- **THEN** the system SHALL ensure the April 2026 partition exists

#### Scenario: Daily job drops expired partition
- **WHEN** LOG_RETENTION_DAYS is 30 and the daily job runs on March 24, 2026
- **THEN** the system SHALL drop the February 2026 partition (older than 30 days) if it exists

#### Scenario: No partition to drop
- **WHEN** the daily job runs and no partitions are older than LOG_RETENTION_DAYS
- **THEN** the system SHALL skip dropping without error

### Requirement: Configurable retention period
The retention period SHALL be configurable via the LOG_RETENTION_DAYS environment variable. If not set, it SHALL default to 30 days.

#### Scenario: Custom retention
- **WHEN** LOG_RETENTION_DAYS is set to 7
- **THEN** partitions older than 7 days SHALL be dropped by the daily maintenance job

#### Scenario: Default retention
- **WHEN** LOG_RETENTION_DAYS is not set
- **THEN** the system SHALL use 30 days as the retention period
