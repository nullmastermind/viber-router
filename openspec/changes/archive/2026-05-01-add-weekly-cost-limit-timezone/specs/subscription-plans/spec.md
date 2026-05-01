## MODIFIED Requirements

### Requirement: Subscription plans table
The system SHALL store subscription plan templates in a `subscription_plans` table with columns: `id` (UUID PK), `name` (TEXT NOT NULL), `sub_type` (TEXT NOT NULL, one of 'fixed', 'hourly_reset', or 'pay_per_request'), `cost_limit_usd` (FLOAT8 NOT NULL), `weekly_cost_limit_usd` (FLOAT8, nullable), `model_limits` (JSONB DEFAULT '{}'), `model_request_costs` (JSONB NOT NULL DEFAULT '{}'), `rpm_limit` (FLOAT8, nullable), `tpm_limit` (FLOAT8, nullable), `reset_hours` (INT, nullable), `duration_days` (INT NOT NULL), `is_active` (BOOLEAN DEFAULT true), `created_at` (TIMESTAMPTZ), `updated_at` (TIMESTAMPTZ).

#### Scenario: Table structure
- **WHEN** the migration runs
- **THEN** the `subscription_plans` table SHALL have all specified columns including `weekly_cost_limit_usd`, `model_request_costs`, `rpm_limit`, and `tpm_limit`, and the `sub_type` CHECK constraint SHALL include 'pay_per_request'

### Requirement: Create subscription plan
The system SHALL allow creating a plan via POST `/api/admin/subscription-plans` with body `{ "name": "<string>", "sub_type": "fixed"|"hourly_reset"|"pay_per_request", "cost_limit_usd": <number>, "weekly_cost_limit_usd": <number|null>, "model_limits": {"<model_name>": <number>}, "model_request_costs": {"<model_name>": <number>}, "rpm_limit": <number|null>, "tpm_limit": <number|null>, "reset_hours": <number|null>, "duration_days": <number> }`.

#### Scenario: Create fixed plan
- **WHEN** an admin sends POST `/api/admin/subscription-plans` with `{ "name": "Pro", "sub_type": "fixed", "cost_limit_usd": 1000.0, "weekly_cost_limit_usd": 100.0, "model_limits": {"claude-opus-4-6": 300.0}, "duration_days": 30 }`
- **THEN** the system SHALL create the plan and return it with status 201 including `weekly_cost_limit_usd: 100.0`

#### Scenario: Create hourly_reset plan
- **WHEN** an admin sends POST `/api/admin/subscription-plans` with `{ "name": "Rate Limited", "sub_type": "hourly_reset", "cost_limit_usd": 100.0, "weekly_cost_limit_usd": null, "reset_hours": 2, "duration_days": 30 }`
- **THEN** the system SHALL create the plan and return it with status 201 including `weekly_cost_limit_usd: null`

#### Scenario: Create pay_per_request plan
- **WHEN** an admin sends POST with `{ "sub_type": "pay_per_request", "model_request_costs": {"claude-sonnet-4-6": 0.10}, "cost_limit_usd": 50.0, "weekly_cost_limit_usd": 20.0, "duration_days": 30 }`
- **THEN** the system SHALL create the plan and return it with status 201 including `weekly_cost_limit_usd: 20.0`

#### Scenario: Create pay_per_request plan with reset window
- **WHEN** an admin sends POST with `{ "sub_type": "pay_per_request", "model_request_costs": {"claude-sonnet-4-6": 0.10}, "reset_hours": 24, "cost_limit_usd": 10.0, "weekly_cost_limit_usd": 5.0, "duration_days": 30 }`
- **THEN** the system SHALL create the plan and return it with status 201

#### Scenario: Create plan with TPM limit
- **WHEN** an admin sends POST `/api/admin/subscription-plans` with `tpm_limit: 100000`
- **THEN** the system SHALL persist `tpm_limit = 100000` and return it in the created plan response

#### Scenario: Create pay_per_request plan with empty model_request_costs
- **WHEN** an admin sends POST with `sub_type: "pay_per_request"` and `model_request_costs: {}`
- **THEN** the system SHALL return 400 with error indicating model_request_costs must not be empty for pay_per_request

#### Scenario: Invalid sub_type
- **WHEN** an admin sends POST with `sub_type` not in ('fixed', 'hourly_reset', 'pay_per_request')
- **THEN** the system SHALL return 400 with error message

#### Scenario: Missing reset_hours for hourly_reset
- **WHEN** an admin sends POST with `sub_type: "hourly_reset"` and `reset_hours` is null
- **THEN** the system SHALL return 400 with error message

## ADDED Requirements

### Requirement: Update subscription plan weekly limit
The system SHALL allow PATCH `/api/admin/subscription-plans/{id}` to update `weekly_cost_limit_usd` independently of other plan fields.

#### Scenario: Update weekly cost limit
- **WHEN** an admin patches a plan with `{ "weekly_cost_limit_usd": 75.0 }`
- **THEN** the system SHALL update the plan and return `weekly_cost_limit_usd: 75.0`

#### Scenario: Clear weekly cost limit
- **WHEN** an admin patches a plan with `{ "weekly_cost_limit_usd": null }`
- **THEN** the system SHALL update the plan and return `weekly_cost_limit_usd: null`
