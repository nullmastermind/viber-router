## MODIFIED Requirements

### Requirement: Key subscriptions table
The system SHALL store subscription instances in a `key_subscriptions` table with columns: `id` (UUID PK), `group_key_id` (UUID FK to group_keys ON DELETE CASCADE), `plan_id` (UUID FK to subscription_plans, nullable), `sub_type` (TEXT NOT NULL), `cost_limit_usd` (FLOAT8 NOT NULL), `model_limits` (JSONB DEFAULT '{}'), `model_request_costs` (JSONB NOT NULL DEFAULT '{}'), `reset_hours` (INT, nullable), `duration_days` (INT NOT NULL), `status` (TEXT NOT NULL DEFAULT 'active'), `activated_at` (TIMESTAMPTZ, nullable), `expires_at` (TIMESTAMPTZ, nullable), `created_at` (TIMESTAMPTZ). Index on `(group_key_id, status)`.

#### Scenario: Table structure
- **WHEN** the migration runs
- **THEN** the `key_subscriptions` table SHALL have all specified columns including `model_request_costs` and the `sub_type` CHECK constraint SHALL include 'pay_per_request'

### Requirement: Assign subscription to sub-key
The system SHALL allow assigning a plan to a sub-key via POST `/api/admin/groups/:group_id/keys/:key_id/subscriptions` with body `{ "plan_id": "<uuid>" }`. The system SHALL snapshot all plan values into the subscription record, including `model_request_costs`.

#### Scenario: Assign plan to sub-key
- **WHEN** an admin sends POST with `{ "plan_id": "<uuid>" }` referencing an active plan
- **THEN** the system SHALL create a `key_subscriptions` record with `sub_type`, `cost_limit_usd`, `model_limits`, `model_request_costs`, `reset_hours`, `duration_days` copied from the plan, `status: "active"`, `activated_at: NULL`, and return it with status 201

#### Scenario: Assign pay_per_request plan snapshots model_request_costs
- **WHEN** an admin assigns a `pay_per_request` plan with `model_request_costs: {"claude-sonnet-4-6": 0.10}`
- **THEN** the created `key_subscriptions` record SHALL have `model_request_costs: {"claude-sonnet-4-6": 0.10}` snapshotted from the plan

#### Scenario: Assign inactive plan
- **WHEN** an admin sends POST referencing a plan with `is_active: false`
- **THEN** the system SHALL return 400 with error "Plan is not active"

#### Scenario: Plan not found
- **WHEN** an admin sends POST with a non-existent plan_id
- **THEN** the system SHALL return 404
