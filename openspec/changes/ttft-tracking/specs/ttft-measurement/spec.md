## ADDED Requirements

### Requirement: TTFT measurement on SSE streaming requests
The system SHALL measure Time to First Token (TTFT) for every SSE streaming response. TTFT is defined as the elapsed time from when the upstream HTTP response headers are received to when the first SSE data chunk arrives from the upstream server.

#### Scenario: Successful TTFT measurement
- **WHEN** the upstream returns HTTP 200 with content-type `text/event-stream` and the first SSE chunk arrives after 450ms
- **THEN** the system SHALL record a TTFT measurement of approximately 450ms for that server

#### Scenario: Non-streaming request ignored
- **WHEN** the upstream returns a non-SSE response (no `text/event-stream` content-type)
- **THEN** the system SHALL NOT record any TTFT measurement

#### Scenario: TTFT recorded even without auto-switch enabled
- **WHEN** a group has `ttft_timeout_ms` set to NULL (disabled) and an SSE response is received
- **THEN** the system SHALL still measure and record the TTFT value

### Requirement: TTFT data persistence
The system SHALL persist TTFT measurements to a `ttft_logs` database table partitioned by `created_at` (monthly, same as proxy_logs). Each record SHALL include: id, created_at, group_id, server_id, request_model, ttft_ms (NULL if timed out), timed_out (boolean), and request_path.

#### Scenario: Successful TTFT persisted
- **WHEN** a TTFT measurement of 380ms is recorded for server S1 in group G1 with model "claude-opus-4-6" on path "/v1/messages"
- **THEN** the system SHALL write a row to `ttft_logs` with ttft_ms=380, timed_out=false, and the corresponding group_id, server_id, request_model, and request_path

#### Scenario: Timed-out TTFT persisted
- **WHEN** a TTFT timeout occurs (first chunk did not arrive within threshold)
- **THEN** the system SHALL write a row to `ttft_logs` with ttft_ms=NULL, timed_out=true

#### Scenario: TTFT persistence uses buffered async writes
- **WHEN** a TTFT measurement is recorded
- **THEN** the system SHALL send it via an mpsc channel to a background flush task that batch-inserts to the database (same pattern as proxy_logs)

#### Scenario: TTFT buffer full
- **WHEN** the TTFT mpsc channel is full
- **THEN** the system SHALL drop the measurement and log a warning (non-blocking, never delays the proxy response)

### Requirement: TTFT auto-switch on timeout
The system SHALL abort an SSE streaming connection and try the next server in the failover waterfall when the first SSE chunk does not arrive within the group's configured `ttft_timeout_ms` threshold. The aborted connection SHALL be closed (TCP connection dropped) to stop upstream processing and billing.

#### Scenario: TTFT timeout triggers failover
- **WHEN** group G1 has `ttft_timeout_ms=3000` and server S1 (priority 1) returns HTTP 200 with SSE headers but no data chunk arrives within 3000ms, and server S2 (priority 2) exists
- **THEN** the system SHALL close the connection to S1 and forward the request to S2

#### Scenario: TTFT timeout on second-to-last server, last server succeeds
- **WHEN** server S1 times out on TTFT, server S2 times out on TTFT, and server S3 is the last server
- **THEN** the system SHALL wait indefinitely for S3's first chunk (no TTFT timeout on last server)

#### Scenario: Single server group skips TTFT timeout
- **WHEN** a group has only one server and `ttft_timeout_ms=3000`
- **THEN** the system SHALL NOT apply TTFT timeout logic and SHALL wait indefinitely for the first chunk

#### Scenario: TTFT disabled (NULL)
- **WHEN** a group has `ttft_timeout_ms` set to NULL
- **THEN** the system SHALL NOT apply any TTFT timeout logic (existing behavior preserved)

#### Scenario: Empty stream after 200
- **WHEN** the upstream returns HTTP 200 with SSE headers but the stream closes immediately without sending any data
- **THEN** the system SHALL treat this as a connection error and try the next server

#### Scenario: First chunk received within timeout
- **WHEN** the first SSE chunk arrives within the `ttft_timeout_ms` threshold
- **THEN** the system SHALL record the TTFT, include the first chunk in the response stream, and continue streaming the rest to the client

### Requirement: TTFT timeout configuration on groups
The `groups` table SHALL have a nullable `ttft_timeout_ms` column (INTEGER). When NULL, TTFT auto-switch is disabled. When set, it defines the maximum milliseconds to wait for the first SSE chunk before trying the next server.

#### Scenario: Group created without TTFT timeout
- **WHEN** a new group is created without specifying `ttft_timeout_ms`
- **THEN** the column SHALL default to NULL (TTFT auto-switch disabled)

#### Scenario: Group TTFT timeout updated via API
- **WHEN** an admin updates a group with `ttft_timeout_ms: 5000`
- **THEN** the group's TTFT timeout SHALL be set to 5000ms and the cached GroupConfig SHALL be invalidated

#### Scenario: Group TTFT timeout cleared
- **WHEN** an admin updates a group with `ttft_timeout_ms: null`
- **THEN** TTFT auto-switch SHALL be disabled for that group

### Requirement: Partition management supports multiple tables
The partition management system SHALL support creating and dropping partitions for any table prefix, not just `proxy_logs`. Both `proxy_logs` and `ttft_logs` SHALL have partitions managed automatically.

#### Scenario: Partitions created for both tables on startup
- **WHEN** the server starts
- **THEN** the system SHALL ensure partitions exist for current and next month for both `proxy_logs` and `ttft_logs`

#### Scenario: Expired partitions dropped for both tables
- **WHEN** the partition cleanup runs
- **THEN** the system SHALL drop expired partitions for both `proxy_logs` and `ttft_logs` based on retention policy
