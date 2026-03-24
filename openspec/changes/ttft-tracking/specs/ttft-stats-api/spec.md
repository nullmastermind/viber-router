## ADDED Requirements

### Requirement: TTFT statistics API endpoint
The system SHALL provide a GET endpoint at `/api/admin/ttft-stats` that returns aggregated TTFT statistics per server for a given group within a time window. The endpoint SHALL require admin authentication.

#### Scenario: Fetch TTFT stats for a group over last hour
- **WHEN** an authenticated admin sends `GET /api/admin/ttft-stats?group_id=<uuid>&period=1h`
- **THEN** the system SHALL return JSON with per-server statistics including: server_id, server_name, avg_ttft_ms, p50_ttft_ms, p95_ttft_ms, timeout_count, total_count, and an array of individual data points (created_at, ttft_ms, timed_out)

#### Scenario: No TTFT data available
- **WHEN** an authenticated admin queries TTFT stats for a group with no TTFT records in the requested period
- **THEN** the system SHALL return an empty `servers` array

#### Scenario: Missing group_id parameter
- **WHEN** the request omits the `group_id` query parameter
- **THEN** the system SHALL return HTTP 400 with an error message

#### Scenario: Unauthenticated request
- **WHEN** a request to `/api/admin/ttft-stats` has no valid admin token
- **THEN** the system SHALL return HTTP 401 (handled by existing admin auth middleware)
