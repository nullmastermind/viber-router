## ADDED Requirements

### Requirement: Log proxy requests with non-200 final status
The system SHALL record a log entry for every proxy request whose final response status code is not 200. The log entry SHALL be created after the failover waterfall completes (either a server responded with a non-failover non-200 status, or all servers were exhausted).

#### Scenario: Upstream returns 429
- **WHEN** a proxy request is routed to server A and server A returns 429 (a failover code), then server B returns 429
- **THEN** the system SHALL create a log entry with status_code=429, error_type="upstream_error", and failover_chain containing both server attempts

#### Scenario: All servers exhausted
- **WHEN** a proxy request exhausts all servers in the group (all return failover status codes or connection errors)
- **THEN** the system SHALL create a log entry with status_code=429, error_type="all_servers_exhausted", and failover_chain containing every server attempt

#### Scenario: Connection error on upstream
- **WHEN** a proxy request fails to connect to a server (connection refused, timeout, DNS failure)
- **THEN** the failover_chain entry for that server SHALL have status=0 and error_type="connection_error"

#### Scenario: Non-failover error from upstream
- **WHEN** a proxy request is routed to server A which returns 400 (not a failover code)
- **THEN** the system SHALL create a log entry with status_code=400, error_type="upstream_error", and failover_chain containing only server A

#### Scenario: Successful request (200) is NOT logged
- **WHEN** a proxy request results in a 200 response (possibly after failover)
- **THEN** the system SHALL NOT create a log entry

### Requirement: Log entry schema
Each log entry SHALL contain: id (UUID), created_at (timestamptz), group_id (UUID), group_api_key (text), server_id (UUID, the final server), server_name (text), request_path (text), request_method (text), status_code (smallint), error_type (text), latency_ms (integer, TTFB from first server attempt to final response), failover_chain (JSONB array), and request_model (text, nullable).

#### Scenario: Log entry contains full context
- **WHEN** a log entry is created for a failed proxy request
- **THEN** the entry SHALL include the group's API key, the final server's name and ID, the HTTP method and path, the upstream status code, total latency in milliseconds, and the full failover chain as a JSONB array

#### Scenario: Request model extracted from body
- **WHEN** the proxy request body contains a JSON "model" field
- **THEN** the log entry's request_model SHALL contain the original model name (before any mapping transformation)

#### Scenario: Request model absent
- **WHEN** the proxy request body does not contain a "model" field or is not valid JSON
- **THEN** the log entry's request_model SHALL be NULL

### Requirement: Async batch log writing
The system SHALL buffer log entries in memory and flush them to PostgreSQL in batches. Flushing SHALL occur every 5 seconds or when the buffer reaches 100 entries, whichever comes first.

#### Scenario: Flush on batch size
- **WHEN** 100 log entries accumulate in the buffer
- **THEN** the system SHALL immediately flush all buffered entries to PostgreSQL in a single batch INSERT

#### Scenario: Flush on timer
- **WHEN** 5 seconds have elapsed since the last flush and the buffer contains at least 1 entry
- **THEN** the system SHALL flush all buffered entries to PostgreSQL

#### Scenario: Empty buffer at timer
- **WHEN** 5 seconds have elapsed and the buffer is empty
- **THEN** the system SHALL NOT execute a database query

### Requirement: Buffer overflow drops entries without blocking
The system SHALL use a bounded buffer (capacity 10,000 entries). When the buffer is full, new log entries SHALL be dropped and a warning SHALL be emitted via tracing::warn. The proxy handler SHALL never block on log writing.

#### Scenario: Buffer full
- **WHEN** the log buffer contains 10,000 entries and a new log entry is produced
- **THEN** the new entry SHALL be dropped, a tracing::warn SHALL be emitted, and the proxy handler SHALL continue without delay

#### Scenario: Database flush failure
- **WHEN** a batch INSERT to PostgreSQL fails
- **THEN** the batch SHALL be dropped, a tracing::warn SHALL be emitted, and the buffer task SHALL continue processing new entries
