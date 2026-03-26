## ADDED Requirements

### Requirement: Subscription plans table
The system SHALL store subscription plan templates in a `subscription_plans` table with columns: `id` (UUID PK), `name` (TEXT NOT NULL), `sub_type` (TEXT NOT NULL, one of 'fixed' or 'hourly_reset'), `cost_limit_usd` (FLOAT8 NOT NULL), `model_limits` (JSONB DEFAULT '{}'), `reset_hours` (INT, nullable), `duration_days` (INT NOT NULL), `is_active` (BOOLEAN DEFAULT true), `created_at` (TIMESTAMPTZ), `updated_at` (TIMESTAMPTZ).

#### Scenario: Table structure
- **WHEN** the migration runs
- **THEN** the `subscription_plans` table SHALL be created with all specified columns and appropriate defaults

### Requirement: Create subscription plan
The system SHALL allow creating a plan via POST `/api/admin/subscription-plans` with body `{ "name": "<string>", "sub_type": "fixed"|"hourly_reset", "cost_limit_usd": <number>, "model_limits": {"<model_name>": <number>}, "reset_hours": <number|null>, "duration_days": <number> }`.

#### Scenario: Create fixed plan
- **WHEN** an admin sends POST `/api/admin/subscription-plans` with `{ "name": "Pro", "sub_type": "fixed", "cost_limit_usd": 1000.0, "model_limits": {"claude-opus-4-6": 300.0}, "duration_days": 30 }`
- **THEN** the system SHALL create the plan and return it with status 201

#### Scenario: Create hourly_reset plan
- **WHEN** an admin sends POST `/api/admin/subscription-plans` with `{ "name": "Rate Limited", "sub_type": "hourly_reset", "cost_limit_usd": 100.0, "reset_hours": 2, "duration_days": 30 }`
- **THEN** the system SHALL create the plan and return it with status 201

#### Scenario: Invalid sub_type
- **WHEN** an admin sends POST with `sub_type` not in ('fixed', 'hourly_reset')
- **THEN** the system SHALL return 400 with error message

#### Scenario: Missing reset_hours for hourly_reset
- **WHEN** an admin sends POST with `sub_type: "hourly_reset"` and `reset_hours` is null
- **THEN** the system SHALL return 400 with error message

### Requirement: List subscription plans
The system SHALL return all plans via GET `/api/admin/subscription-plans`. Plans with `is_active: false` SHALL be included but distinguishable.

#### Scenario: List all plans
- **WHEN** an admin sends GET `/api/admin/subscription-plans`
- **THEN** the system SHALL return all plans ordered by `created_at` descending

### Requirement: Update subscription plan
The system SHALL allow updating a plan via PATCH `/api/admin/subscription-plans/:id` with partial fields. Changes do NOT affect existing subscriptions (snapshot pattern).

#### Scenario: Update plan name
- **WHEN** an admin sends PATCH with `{ "name": "Pro Plus" }`
- **THEN** the system SHALL update the plan name and return the updated plan

#### Scenario: Disable plan
- **WHEN** an admin sends PATCH with `{ "is_active": false }`
- **THEN** the plan SHALL no longer appear in the active plans dropdown but existing subscriptions are unaffected

### Requirement: Delete subscription plan
The system SHALL allow deleting a plan via DELETE `/api/admin/subscription-plans/:id` only if no key_subscriptions reference it.

#### Scenario: Delete unreferenced plan
- **WHEN** an admin sends DELETE for a plan with no subscriptions
- **THEN** the system SHALL delete the plan and return 204

#### Scenario: Delete referenced plan
- **WHEN** an admin sends DELETE for a plan that has subscriptions
- **THEN** the system SHALL return 409 with error "Plan has existing subscriptions"
