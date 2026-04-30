## MODIFIED Requirements

### Requirement: Bonus subscription creation without plan
The system SHALL allow creating a bonus subscription directly (without a `plan_id`) via POST `/api/admin/groups/:group_id/keys/:key_id/subscriptions`. The request body MUST include `bonus_name`, `bonus_base_url`, and `bonus_api_key`. Optional fields are `bonus_quota_url`, `bonus_quota_headers`, and `bonus_allowed_models`. When provided, `bonus_allowed_models` MUST be stored as model name strings. A missing, null, or empty `bonus_allowed_models` value SHALL mean the bonus subscription accepts all models. The system SHALL set `sub_type = 'bonus'`, `cost_limit_usd = 0`, `model_limits = {}`, `model_request_costs = {}`, `duration_days = 36500`, `reset_hours = NULL`, `rpm_limit = NULL`, `plan_id = NULL` for all bonus subscriptions.

#### Scenario: Create bonus with required fields
- **WHEN** an admin POSTs `{ "bonus_name": "Claude Max", "bonus_base_url": "https://api.anthropic.com", "bonus_api_key": "sk-ant-xxx" }` to the subscriptions endpoint
- **THEN** the system SHALL create a `key_subscriptions` record with `sub_type = 'bonus'`, the provided bonus fields set, unrestricted model handling, and return it with status 201

#### Scenario: Create bonus with optional fields
- **WHEN** an admin POSTs bonus creation body including `bonus_quota_url` and `bonus_quota_headers`
- **THEN** the system SHALL store `bonus_quota_url` and `bonus_quota_headers` in the created record

#### Scenario: Create bonus with allowed models
- **WHEN** an admin POSTs bonus creation body including `bonus_allowed_models = ["claude-sonnet-4-5"]`
- **THEN** the system SHALL store `bonus_allowed_models` on the created record as the provided model name strings

#### Scenario: Bonus creation missing required field
- **WHEN** an admin POSTs `{ "bonus_name": "Claude Max" }` without `bonus_base_url` or `bonus_api_key`
- **THEN** the system SHALL return HTTP 400 with a validation error

#### Scenario: Request has neither plan_id nor bonus fields
- **WHEN** an admin POSTs `{}` with no `plan_id` and no bonus fields
- **THEN** the system SHALL return HTTP 400 with an error indicating plan_id or bonus fields are required

### Requirement: Bonus server list in SubCheckResult
The subscription engine's `check_subscriptions()` function SHALL separate bonus subscriptions from non-bonus subscriptions. If any active bonus subscriptions are eligible for the request model, the function SHALL return `BonusServers { servers, fallback_subscription }`. `servers` SHALL be the list of active bonus subs that accept the request model ordered by `created_at ASC`, each as a `BonusServer { subscription_id, base_url, api_key, name, allowed_models }`. `allowed_models` SHALL contain the stored bonus model allowlist or an empty list for unrestricted bonus subscriptions. `fallback_subscription` SHALL be `Some((sub_id, rpm_limit))` if the non-bonus check would have returned `Allowed`, or `None` otherwise.

#### Scenario: Sub-key has one bonus sub and one fixed sub with budget
- **WHEN** `check_subscriptions()` is called for a sub-key with one active bonus sub that accepts the request model and one active fixed sub with remaining budget
- **THEN** the function SHALL return `BonusServers { servers: [bonus], fallback_subscription: Some((fixed_sub_id, None)) }`

#### Scenario: Sub-key has only bonus subs
- **WHEN** `check_subscriptions()` is called for a sub-key with only active bonus subscriptions that accept the request model
- **THEN** the function SHALL return `BonusServers { servers: [bonus1, bonus2, ...], fallback_subscription: None }`

#### Scenario: Sub-key has bonus subs and exhausted non-bonus subs
- **WHEN** all non-bonus subscriptions are exhausted or expired and at least one bonus sub accepts the request model
- **THEN** the function SHALL return `BonusServers { servers: [bonus], fallback_subscription: None }`

#### Scenario: Sub-key has no eligible bonus subs
- **WHEN** `check_subscriptions()` is called for a sub-key with no active bonus subscriptions that accept the request model
- **THEN** the function SHALL NOT return `BonusServers`; existing Allowed/Blocked logic is unchanged

#### Scenario: Sub-key has no bonus subs
- **WHEN** `check_subscriptions()` is called for a sub-key with no bonus subscriptions
- **THEN** the function SHALL NOT return `BonusServers`; existing Allowed/Blocked logic is unchanged

### Requirement: Add Bonus button in admin UI
The admin subscriptions section in `GroupDetailPage.vue` SHALL include an "Add Bonus" button alongside the existing "Add Subscription" button. Clicking "Add Bonus" SHALL open a dialog with fields: Name (required), Base URL (required, default "https://api.anthropic.com"), API Key (required), Quota Check URL (optional), Quota Headers (optional JSON text), and Allowed Models (optional multi-select). Allowed Models options SHALL come from the group's allowed models and SHALL submit selected model names. Submitting the dialog SHALL POST the bonus fields to the subscriptions endpoint. The subscription table SHALL render bonus rows with `bonus_name` in the Plan column, "Bonus" in the Type column, allowed models shown as selected names or "All models", and omit cost/budget columns.

#### Scenario: Open Add Bonus dialog
- **WHEN** the admin clicks the "Add Bonus" button in the subscriptions section
- **THEN** a dialog opens with Name, Base URL, API Key, Quota Check URL, Quota Headers, and Allowed Models fields

#### Scenario: Submit Add Bonus dialog
- **WHEN** the admin fills required fields, optionally selects allowed models, and submits the dialog
- **THEN** the system POSTs `{ "bonus_name": "...", "bonus_base_url": "...", "bonus_api_key": "...", "bonus_allowed_models": [...] }` to the subscriptions endpoint and adds the new row to the table

#### Scenario: Bonus row display in subscription table
- **WHEN** a subscription row has `sub_type = 'bonus'`
- **THEN** the Plan column shows `bonus_name`, the Type column shows "Bonus", allowed models show configured model names or "All models", and cost/budget columns show "N/A" or are omitted
