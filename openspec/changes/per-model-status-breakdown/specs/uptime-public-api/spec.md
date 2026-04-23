## MODIFIED Requirements

### Requirement: Chain-level uptime endpoint
The system SHALL provide `GET /api/public/uptime?key=...` returning chain-level uptime data bucketed into 90 x 30-minute intervals, plus a `models` array with per-model uptime data. The endpoint SHALL use the same key validation and IP rate limiting as the existing public usage endpoint.

#### Scenario: Successful response
- **WHEN** a user sends GET `/api/public/uptime?key=sk-vibervn-abc123` with a valid active sub-key
- **THEN** the system SHALL return HTTP 200 with a JSON object containing `status` (string: "operational", "degraded", "down", or "no_data"), `uptime_percent` (float, last 30 minutes), `buckets` array of 90 `{ timestamp, total_requests, successful_requests }` objects sorted chronologically, and `models` array of per-model entries each containing `{ model, status, uptime_percent, buckets }`

#### Scenario: Chain success definition
- **WHEN** a proxy request (identified by request_id) has attempts with status codes [429, 200]
- **THEN** the request SHALL be counted as successful because at least one attempt returned a 2xx status code

#### Scenario: Chain failure definition
- **WHEN** a proxy request (identified by request_id) has attempts with status codes [429, 500, 0]
- **THEN** the request SHALL be counted as failed because no attempt returned a 2xx status code

#### Scenario: Status text derivation
- **WHEN** the most recent 30-minute bucket has >95% successful requests
- **THEN** the `status` field SHALL be "operational"

#### Scenario: Degraded status
- **WHEN** the most recent 30-minute bucket has 50-95% successful requests
- **THEN** the `status` field SHALL be "degraded"

#### Scenario: Down status
- **WHEN** the most recent 30-minute bucket has <50% successful requests
- **THEN** the `status` field SHALL be "down"

#### Scenario: No data status
- **WHEN** the most recent 30-minute bucket has 0 requests
- **THEN** the `status` field SHALL be "no_data"

#### Scenario: Invalid key
- **WHEN** a user sends a request with an invalid or inactive sub-key
- **THEN** the system SHALL return HTTP 403

#### Scenario: Rate limited
- **WHEN** a user exceeds the IP rate limit
- **THEN** the system SHALL return HTTP 429 with Retry-After header
