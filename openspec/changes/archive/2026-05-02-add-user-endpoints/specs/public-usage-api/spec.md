## MODIFIED Requirements

### Requirement: Public usage endpoint
The system SHALL expose `GET /api/public/usage?key=<sub-key>` that accepts a sub-key and returns usage, subscription data, and user endpoint data without admin authentication. For bonus subscriptions, the response SHALL include `bonus_name`, `bonus_quotas` (fetched from `bonus_quota_url` if set, null otherwise), and `bonus_usage` (per-model request counts from the last 30 days). For user endpoints, the response SHALL include endpoint identity/configuration needed by the public page, quota data when configured, enabled state, priority mode, and per-model usage statistics from the last 30 days.

#### Scenario: Valid active sub-key
- **WHEN** a request is made with a valid, active sub-key
- **THEN** the system returns HTTP 200 with JSON containing `key_name`, `group_name`, `usage` (array of per-model aggregations), `subscriptions` (array of all subscriptions with `cost_used`), and `user_endpoints` (array of endpoint entries owned by the sub-key)

#### Scenario: Invalid sub-key
- **WHEN** a request is made with a sub-key that does not exist in `group_keys`
- **THEN** the system returns HTTP 403 with `{"error": "Invalid or inactive key"}`

#### Scenario: Deactivated sub-key
- **WHEN** a request is made with a sub-key that exists but has `is_active = false`
- **THEN** the system returns HTTP 403 with `{"error": "Invalid or inactive key"}` (same response as non-existent key)

#### Scenario: Missing key parameter
- **WHEN** a request is made without the `key` query parameter
- **THEN** the system returns HTTP 400 with `{"error": "key parameter is required"}`

#### Scenario: Sub-key with bonus subscription — quota URL set
- **WHEN** a sub-key has a bonus subscription with `bonus_quota_url` set and the URL returns valid quota data
- **THEN** the subscription entry includes `bonus_name`, non-null `bonus_quotas` array, and `bonus_usage` per-model counts

#### Scenario: Sub-key with bonus subscription — quota URL not set
- **WHEN** a sub-key has a bonus subscription with `bonus_quota_url = NULL`
- **THEN** the subscription entry includes `bonus_name`, `bonus_quotas: null`, and `bonus_usage` per-model counts

#### Scenario: Sub-key with user endpoints
- **WHEN** a valid active sub-key owns user endpoints
- **THEN** the response includes those endpoints in `user_endpoints` with `id`, `name`, `base_url`, `model_mappings`, `quota_url`, `quota_headers`, `priority_mode`, `is_enabled`, quota data when configured, and 30-day per-model endpoint usage

#### Scenario: Sub-key without user endpoints
- **WHEN** a valid active sub-key owns no user endpoints
- **THEN** the response includes `user_endpoints: []`
