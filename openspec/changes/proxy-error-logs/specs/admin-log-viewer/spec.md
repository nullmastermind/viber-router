## ADDED Requirements

### Requirement: Log query API endpoint
The system SHALL provide a GET /api/admin/logs endpoint that returns proxy error logs with server-side pagination and filtering. The endpoint SHALL be protected by admin authentication.

#### Scenario: Query logs without filters
- **WHEN** an authenticated admin sends GET /api/admin/logs
- **THEN** the system SHALL return the most recent 20 log entries ordered by created_at DESC, with a cursor for the next page

#### Scenario: Filter by status code
- **WHEN** an authenticated admin sends GET /api/admin/logs?status_code=429
- **THEN** the system SHALL return only log entries with status_code=429

#### Scenario: Filter by group
- **WHEN** an authenticated admin sends GET /api/admin/logs?group_id=uuid
- **THEN** the system SHALL return only log entries for the specified group

#### Scenario: Filter by server
- **WHEN** an authenticated admin sends GET /api/admin/logs?server_id=uuid
- **THEN** the system SHALL return only log entries where server_id matches (the final responding server)

#### Scenario: Filter by date range
- **WHEN** an authenticated admin sends GET /api/admin/logs?from=2026-03-20T00:00:00Z&to=2026-03-24T23:59:59Z
- **THEN** the system SHALL return only log entries within the specified date range

#### Scenario: Search by API key
- **WHEN** an authenticated admin sends GET /api/admin/logs?api_key=sk-vibervn-abc
- **THEN** the system SHALL return only log entries where group_api_key matches the search term (exact match)

#### Scenario: Cursor-based pagination
- **WHEN** an authenticated admin sends GET /api/admin/logs?cursor=2026-03-24T14:23:01Z&page_size=50
- **THEN** the system SHALL return up to 50 log entries with created_at before the cursor value, ordered by created_at DESC

#### Scenario: Combined filters
- **WHEN** an authenticated admin sends GET /api/admin/logs?status_code=429&group_id=uuid&from=2026-03-20T00:00:00Z
- **THEN** the system SHALL return log entries matching ALL specified filters

### Requirement: Log statistics endpoint
The system SHALL provide a GET /api/admin/logs/stats endpoint that returns aggregate counts for the current filter criteria.

#### Scenario: Stats without filters
- **WHEN** an authenticated admin sends GET /api/admin/logs/stats
- **THEN** the system SHALL return total log count and count per status code for the last 24 hours

#### Scenario: Stats with filters
- **WHEN** an authenticated admin sends GET /api/admin/logs/stats?group_id=uuid&from=2026-03-20T00:00:00Z
- **THEN** the system SHALL return total count and per-status-code counts matching the filters

### Requirement: Admin UI log viewer page
The admin frontend SHALL include a Logs page accessible from the main navigation. The page SHALL display proxy error logs in a table with filters and expandable row detail.

#### Scenario: Page loads with recent logs
- **WHEN** an admin navigates to the Logs page
- **THEN** the page SHALL display a table of recent proxy error logs with columns: Time, Status, Group, Server, Model, Latency

#### Scenario: Loading state
- **WHEN** logs are being fetched from the API
- **THEN** the page SHALL display a loading spinner

#### Scenario: Empty state
- **WHEN** no logs match the current filters
- **THEN** the page SHALL display "No logs matching filters" with a suggestion to adjust filters

#### Scenario: Error state
- **WHEN** the API request fails
- **THEN** the page SHALL display an error message "Failed to load logs" with a retry button

#### Scenario: Filter by status code
- **WHEN** an admin selects a status code from the status filter dropdown
- **THEN** the table SHALL reload showing only logs with that status code

#### Scenario: Filter by group
- **WHEN** an admin selects a group from the group filter dropdown
- **THEN** the table SHALL reload showing only logs for that group

#### Scenario: Filter by server
- **WHEN** an admin selects a server from the server filter dropdown
- **THEN** the table SHALL reload showing only logs for that server

#### Scenario: Filter by date range
- **WHEN** an admin selects a from and to date
- **THEN** the table SHALL reload showing only logs within that date range

#### Scenario: Search by API key
- **WHEN** an admin types an API key into the search field and presses Enter or clicks search
- **THEN** the table SHALL reload showing only logs matching that API key

#### Scenario: Expand row to see failover chain
- **WHEN** an admin clicks on a log row
- **THEN** the row SHALL expand to show the full failover chain with each server attempt (server name, status code, latency), the full API key, request path, and model mapping info

#### Scenario: Server-side pagination
- **WHEN** more logs exist than the page size
- **THEN** the table SHALL show pagination controls and load pages from the server on navigation
