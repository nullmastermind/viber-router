## ADDED Requirements

### Requirement: Bonus subscription creation without plan
The system SHALL allow creating a bonus subscription directly (without a `plan_id`) via POST `/api/admin/groups/:group_id/keys/:key_id/subscriptions`. The request body MUST include `bonus_name`, `bonus_base_url`, and `bonus_api_key`. Optional fields are `bonus_quota_url` and `bonus_quota_headers`. The system SHALL set `sub_type = 'bonus'`, `cost_limit_usd = 0`, `model_limits = {}`, `model_request_costs = {}`, `duration_days = 36500`, `reset_hours = NULL`, `rpm_limit = NULL`, `plan_id = NULL` for all bonus subscriptions.

#### Scenario: Create bonus with required fields
- **WHEN** an admin POSTs `{ "bonus_name": "Claude Max", "bonus_base_url": "https://api.anthropic.com", "bonus_api_key": "sk-ant-xxx" }` to the subscriptions endpoint
- **THEN** the system SHALL create a `key_subscriptions` record with `sub_type = 'bonus'`, the provided bonus fields set, and return it with status 201

#### Scenario: Create bonus with optional fields
- **WHEN** an admin POSTs bonus creation body including `bonus_quota_url` and `bonus_quota_headers`
- **THEN** the system SHALL store `bonus_quota_url` and `bonus_quota_headers` in the created record

#### Scenario: Bonus creation missing required field
- **WHEN** an admin POSTs `{ "bonus_name": "Claude Max" }` without `bonus_base_url` or `bonus_api_key`
- **THEN** the system SHALL return HTTP 400 with a validation error

#### Scenario: Request has neither plan_id nor bonus fields
- **WHEN** an admin POSTs `{}` with no `plan_id` and no bonus fields
- **THEN** the system SHALL return HTTP 400 with an error indicating plan_id or bonus fields are required

### Requirement: Bonus subscription routing in proxy
When a sub-key's subscription check returns `BonusServers`, the proxy SHALL try each bonus server in FIFO order (oldest `created_at` first) before the group server waterfall. For each bonus server, the proxy SHALL build the upstream URL as `bonus_base_url.trim_end_matches('/') + "/v1/messages"`, set headers `x-api-key: bonus_api_key` and `authorization: Bearer bonus_api_key`, and forward the transformed request body. A 2xx response from a bonus server SHALL be returned immediately. A non-2xx response SHALL be logged and the proxy SHALL continue to the next bonus server.

#### Scenario: Bonus server returns 2xx
- **WHEN** the first bonus server returns HTTP 200
- **THEN** the proxy SHALL return the response to the client immediately without trying group servers

#### Scenario: Bonus server returns non-2xx, fallback to next bonus
- **WHEN** the first bonus server returns HTTP 529 and a second bonus server exists
- **THEN** the proxy SHALL log the failure and attempt the second bonus server

#### Scenario: All bonus servers exhausted, fallback subscription exists
- **WHEN** all bonus servers return non-2xx and `fallback_subscription` is Some(sub_id, rpm_limit)
- **THEN** the proxy SHALL set `selected_subscription_id = sub_id` and proceed to the group server waterfall as if the sub-key had a normal subscription

#### Scenario: All bonus servers exhausted, no fallback subscription
- **WHEN** all bonus servers return non-2xx and `fallback_subscription` is None
- **THEN** the proxy SHALL proceed to the group server waterfall without subscription tracking (unlimited access to group servers)

#### Scenario: Bonus usage logging — cost is NULL
- **WHEN** a bonus server returns HTTP 200 and token usage is recorded
- **THEN** `token_usage_logs.cost_usd` SHALL be NULL and `token_usage_logs.subscription_id` SHALL be set to the bonus subscription's UUID

### Requirement: Bonus server list in SubCheckResult
The subscription engine's `check_subscriptions()` function SHALL separate bonus subscriptions from non-bonus subscriptions. If any active bonus subscriptions exist, the function SHALL return `BonusServers { servers, fallback_subscription }`. `servers` SHALL be the list of active bonus subs ordered by `created_at ASC`, each as a `BonusServer { subscription_id, base_url, api_key, name }`. `fallback_subscription` SHALL be `Some((sub_id, rpm_limit))` if the non-bonus check would have returned `Allowed`, or `None` otherwise.

#### Scenario: Sub-key has one bonus sub and one fixed sub with budget
- **WHEN** `check_subscriptions()` is called for a sub-key with one active bonus sub and one active fixed sub with remaining budget
- **THEN** the function SHALL return `BonusServers { servers: [bonus], fallback_subscription: Some((fixed_sub_id, None)) }`

#### Scenario: Sub-key has only bonus subs
- **WHEN** `check_subscriptions()` is called for a sub-key with only active bonus subscriptions
- **THEN** the function SHALL return `BonusServers { servers: [bonus1, bonus2, ...], fallback_subscription: None }`

#### Scenario: Sub-key has bonus subs and exhausted non-bonus subs
- **WHEN** all non-bonus subscriptions are exhausted or expired and at least one bonus sub is active
- **THEN** the function SHALL return `BonusServers { servers: [bonus], fallback_subscription: None }`

#### Scenario: Sub-key has no bonus subs
- **WHEN** `check_subscriptions()` is called for a sub-key with no bonus subscriptions
- **THEN** the function SHALL NOT return `BonusServers`; existing Allowed/Blocked logic is unchanged

### Requirement: Bonus quota fetch in public usage API
For each bonus subscription returned by the public usage API, if `bonus_quota_url` is set, the backend SHALL make an HTTP GET request to that URL with `bonus_quota_headers` (if set) and a 5-second timeout. The response SHALL be parsed as `{ "quotas": [{ "name": "...", "utilization": 0.0 to 1.0, "reset_at": "...", "description": "..." }] }`. On any error (timeout, connection failure, non-2xx, parse error), the backend SHALL return an empty quotas array without failing the overall public usage response.

#### Scenario: Quota URL returns valid response
- **WHEN** a bonus subscription has `bonus_quota_url` set and the URL returns `{ "quotas": [{ "name": "Monthly", "utilization": 0.42, "reset_at": "2026-05-01T00:00:00Z", "description": "Monthly quota" }] }`
- **THEN** the API response includes `bonus_quotas: [{ "name": "Monthly", "utilization": 0.42, "reset_at": "2026-05-01T00:00:00Z", "description": "Monthly quota" }]`

#### Scenario: Quota URL times out
- **WHEN** the quota URL does not respond within 5 seconds
- **THEN** `bonus_quotas` SHALL be an empty array and the overall API response SHALL succeed

#### Scenario: Quota URL not set
- **WHEN** a bonus subscription has `bonus_quota_url = NULL`
- **THEN** `bonus_quotas` SHALL be `null`

### Requirement: Bonus subscription card in public usage page
The public usage page SHALL render bonus subscriptions with a distinct card style. The card SHALL show `bonus_name` as the title with a lightning bolt icon. If `bonus_quotas` is a non-empty array, each quota entry SHALL be rendered as a progress bar (q-linear-progress) showing the utilization percentage, the quota name, and a reset countdown if `reset_at` is present. If `bonus_quotas` is an empty array, the card SHALL show "Quota info unavailable". The card SHALL show `bonus_usage` as a list of "model: count" entries for the last 30 days.

#### Scenario: Bonus card with quota data
- **WHEN** a bonus subscription has quotas and usage data
- **THEN** the card shows: title (bonus_name + lightning icon), one progress bar per quota with utilization %, quota name, and reset countdown if reset_at present, and a list of model/request-count pairs

#### Scenario: Bonus card with empty quotas
- **WHEN** `bonus_quotas` is an empty array
- **THEN** the card shows "Quota info unavailable" in the quota section but still shows bonus_usage

#### Scenario: Bonus card with null quotas (no quota URL configured)
- **WHEN** `bonus_quotas` is null
- **THEN** the quota section is not rendered

### Requirement: Add Bonus button in admin UI
The admin subscriptions section in `GroupDetailPage.vue` SHALL include an "Add Bonus" button alongside the existing "Add Subscription" button. Clicking "Add Bonus" SHALL open a dialog with fields: Name (required), Base URL (required, default "https://api.anthropic.com"), API Key (required), Quota Check URL (optional), Quota Headers (optional JSON text). Submitting the dialog SHALL POST the bonus fields to the subscriptions endpoint. The subscription table SHALL render bonus rows with `bonus_name` in the Plan column, "Bonus" in the Type column, and omit cost/budget columns.

#### Scenario: Open Add Bonus dialog
- **WHEN** the admin clicks the "Add Bonus" button in the subscriptions section
- **THEN** a dialog opens with Name, Base URL, API Key, Quota Check URL, and Quota Headers fields

#### Scenario: Submit Add Bonus dialog
- **WHEN** the admin fills required fields and submits the dialog
- **THEN** the system POSTs `{ "bonus_name": "...", "bonus_base_url": "...", "bonus_api_key": "...", ... }` to the subscriptions endpoint and adds the new row to the table

#### Scenario: Bonus row display in subscription table
- **WHEN** a subscription row has `sub_type = 'bonus'`
- **THEN** the Plan column shows `bonus_name`, the Type column shows "Bonus", and cost/budget columns show "N/A" or are omitted

### Requirement: useSubscriptionType composable supports bonus
The `useSubscriptionType` composable SHALL return label "Bonus", tooltip "External API subscription with its own server. Tried before group servers.", and include a `bonus` option in `getSubTypeOptions()`.

#### Scenario: getSubTypeLabel for bonus
- **WHEN** `getSubTypeLabel('bonus')` is called
- **THEN** it SHALL return "Bonus"

#### Scenario: getSubTypeTooltip for bonus
- **WHEN** `getSubTypeTooltip('bonus')` is called
- **THEN** it SHALL return "External API subscription with its own server. Tried before group servers."
