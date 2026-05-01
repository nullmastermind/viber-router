## Purpose
TBD

## Requirements
### Requirement: Per-server uptime bars endpoint
The system SHALL provide `GET /api/admin/groups/{id}/uptime` returning per-server uptime data bucketed into 90 × 30-minute intervals covering the last 45 hours. The endpoint SHALL require admin authentication.

#### Scenario: Successful response
- **WHEN** an authenticated admin sends GET `/api/admin/groups/{id}/uptime`
- **THEN** the system SHALL return HTTP 200 with a JSON object containing `servers` array, where each entry has `server_id`, `server_name`, and `buckets` array of `{ timestamp, total, success }` objects sorted chronologically

#### Scenario: No uptime data
- **WHEN** an authenticated admin requests uptime for a group with no traffic in the last 45 hours
- **THEN** the system SHALL return HTTP 200 with `servers` array where each server has 90 buckets all with `total: 0, success: 0`

#### Scenario: Group not found
- **WHEN** an authenticated admin requests uptime for a non-existent group ID
- **THEN** the system SHALL return HTTP 404

### Requirement: Uptime percentage calculation
Each bucket SHALL include a success rate calculated as: count of entries with status_code 200-299 divided by total entries. Buckets with zero entries SHALL have `total: 0` and `success: 0`.

#### Scenario: Mixed results in a bucket
- **WHEN** a 30-minute bucket contains 100 entries, 92 with status 200 and 8 with status 429
- **THEN** the bucket SHALL report `total: 100, success: 92`
