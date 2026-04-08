## ADDED Requirements

### Requirement: Public usage endpoint
The system SHALL expose `GET /api/public/usage?key=<sub-key>` that accepts a sub-key and returns usage and subscription data without admin authentication.

#### Scenario: Valid active sub-key
- **WHEN** a request is made with a valid, active sub-key
- **THEN** the system returns HTTP 200 with JSON containing `key_name`, `group_name`, `usage` (array of per-model aggregations), and `subscriptions` (array of all subscriptions with `cost_used`)

#### Scenario: Invalid sub-key
- **WHEN** a request is made with a sub-key that does not exist in `group_keys`
- **THEN** the system returns HTTP 403 with `{"error": "Invalid or inactive key"}`

#### Scenario: Deactivated sub-key
- **WHEN** a request is made with a sub-key that exists but has `is_active = false`
- **THEN** the system returns HTTP 403 with `{"error": "Invalid or inactive key"}` (same response as non-existent key)

#### Scenario: Missing key parameter
- **WHEN** a request is made without the `key` query parameter
- **THEN** the system returns HTTP 400 with `{"error": "key parameter is required"}`

### Requirement: Usage data excludes server information and cache columns
The system SHALL aggregate token usage by model only. The response MUST NOT include `server_id`, `server_name`, or any information that identifies upstream providers. Cache columns SHALL be collapsed into `effective_input_tokens`.

#### Scenario: Usage response shape
- **WHEN** a valid sub-key has usage data
- **THEN** each usage entry contains only: `model`, `effective_input_tokens`, `total_output_tokens`, `request_count`, `cost_usd`

#### Scenario: effective_input_tokens calculation
- **WHEN** a model row has `input_tokens = 1000`, `cache_creation_tokens = 200`, `cache_read_tokens = 500`
- **THEN** `effective_input_tokens` SHALL be `1000 + 200 + 500 = 1700`

#### Scenario: NULL cache tokens
- **WHEN** a model row has `cache_creation_tokens = NULL` and `cache_read_tokens = NULL`
- **THEN** `effective_input_tokens` SHALL equal `input_tokens` (COALESCE to 0)

### Requirement: Usage period is 30 days
The system SHALL return token usage data from the last 30 days only.

#### Scenario: Usage within 30-day window
- **WHEN** a sub-key has usage records from 15 days ago and 45 days ago
- **THEN** only the 15-day-old records are included in the response

### Requirement: Subscription data includes cost_used and window reset time
The system SHALL return non-cancelled subscriptions (`active`, `exhausted`, `expired`) for the sub-key, each enriched with `cost_used` (current cost from Redis/DB) and `window_reset_at` (for subscriptions with `reset_hours` set, when an active window exists). Subscriptions with status `cancelled` are excluded.

#### Scenario: Active fixed subscription
- **WHEN** a sub-key has an active fixed subscription with $2.50 used of $10.00 limit
- **THEN** the subscription entry includes `cost_used: 2.50`, `cost_limit_usd: 10.00`, `window_reset_at: null`

#### Scenario: Active hourly_reset subscription with active window
- **WHEN** a sub-key has an active hourly_reset subscription with `reset_hours: 4` and `sub_window_start:{sub_id}` exists in Redis with epoch `ws`
- **THEN** the subscription entry includes `cost_used` for the current window and `window_reset_at` equal to `DateTime::from_timestamp(ws) + 4 hours` formatted as ISO 8601

#### Scenario: Active hourly_reset subscription with no active window
- **WHEN** a sub-key has an active hourly_reset subscription with `reset_hours: 4` and `sub_window_start:{sub_id}` does not exist in Redis
- **THEN** the subscription entry includes `cost_used: 0.0` and `window_reset_at: null`

#### Scenario: Expired/exhausted subscriptions
- **WHEN** a sub-key has subscriptions with status `expired` or `exhausted`
- **THEN** these subscriptions are included in the response with their actual status and `cost_used: 0.0`

#### Scenario: Cancelled subscriptions are excluded
- **WHEN** a sub-key has subscriptions with status `cancelled`
- **THEN** these subscriptions are NOT included in the response

### Requirement: IP-based rate limiting
The system SHALL rate-limit the public usage endpoint to 30 requests per 60-second window per client IP address using Redis.

#### Scenario: Within rate limit
- **WHEN** a client IP has made fewer than 30 requests in the current 60-second window
- **THEN** the request is processed normally

#### Scenario: Rate limit exceeded
- **WHEN** a client IP has made 30 or more requests in the current 60-second window
- **THEN** the system returns HTTP 429 with `{"error": "Too many requests"}` and `Retry-After: 60` header

#### Scenario: Redis unavailable
- **WHEN** Redis is unavailable for rate limit checking
- **THEN** the system fails open and processes the request normally (consistent with existing rate limiter behavior)
