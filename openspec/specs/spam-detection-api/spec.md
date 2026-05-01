## Purpose
TBD

## Requirements
### Requirement: Low-token spam detection
The system SHALL detect group keys that send an excessive number of requests with very low input token counts within a recent time window.

#### Scenario: Key flagged for low-token spam
- **WHEN** a group key has sent 10 or more requests with `input_tokens < 50` within the last 20 minutes
- **THEN** the system SHALL include that key in the spam detection results with `spam_type: "low_token"` and the actual request count

#### Scenario: Key below threshold not flagged
- **WHEN** a group key has sent fewer than 10 requests with `input_tokens < 50` within the last 20 minutes
- **THEN** the system SHALL NOT include that key in the low-token spam results

### Requirement: Duplicate-request spam detection
The system SHALL detect group keys that send identical request bodies repeatedly within a recent time window.

#### Scenario: Key flagged for duplicate-request spam
- **WHEN** a group key has sent 10 or more requests with the same `content_hash` within the last 10 minutes
- **THEN** the system SHALL include that key in the spam detection results with `spam_type: "duplicate_request"` and the actual request count

#### Scenario: Key below duplicate threshold not flagged
- **WHEN** a group key has sent fewer than 10 requests with the same `content_hash` within the last 10 minutes
- **THEN** the system SHALL NOT include that key in the duplicate-request spam results

#### Scenario: Null content_hash excluded from duplicate detection
- **WHEN** a token usage log entry has `content_hash IS NULL`
- **THEN** the system SHALL NOT consider that entry for duplicate-request spam detection

### Requirement: Spam detection endpoint
The system SHALL expose `GET /api/admin/spam-detection?group_id=<uuid>&page=<int>&limit=<int>` returning paginated spam detection results for a group.

#### Scenario: Results returned for group
- **WHEN** a valid `group_id` is provided
- **THEN** the system SHALL return a `PaginatedResponse` containing spam result items, total count, page, and limit

#### Scenario: Results include full API key
- **WHEN** a key is flagged as spam
- **THEN** the result item SHALL include the full unmasked `api_key` and `name` from the `group_keys` table

#### Scenario: Results include peak RPM
- **WHEN** a key is flagged as spam
- **THEN** the result item SHALL include `peak_rpm` — the maximum number of requests that key made in any single 1-minute bucket (using `date_trunc('minute', created_at)`) within the detection period

#### Scenario: Results include detection metadata
- **WHEN** a key is flagged as spam
- **THEN** the result item SHALL include `spam_type` ("low_token" or "duplicate_request"), `request_count`, and `detected_at` (current server timestamp)

#### Scenario: Pagination applied
- **WHEN** `page` and `limit` query parameters are provided
- **THEN** the system SHALL return only the requested page of results with correct `total`, `page`, and `limit` fields

#### Scenario: Default pagination
- **WHEN** `page` and `limit` are omitted
- **THEN** the system SHALL default to `page=1` and `limit=20`
