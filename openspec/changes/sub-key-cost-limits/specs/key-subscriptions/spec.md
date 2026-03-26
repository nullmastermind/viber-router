## ADDED Requirements

### Requirement: Key subscriptions table
The system SHALL store subscription instances in a `key_subscriptions` table with columns: `id` (UUID PK), `group_key_id` (UUID FK to group_keys ON DELETE CASCADE), `plan_id` (UUID FK to subscription_plans, nullable), `sub_type` (TEXT NOT NULL), `cost_limit_usd` (FLOAT8 NOT NULL), `model_limits` (JSONB DEFAULT '{}'), `reset_hours` (INT, nullable), `duration_days` (INT NOT NULL), `status` (TEXT NOT NULL DEFAULT 'active'), `activated_at` (TIMESTAMPTZ, nullable), `expires_at` (TIMESTAMPTZ, nullable), `created_at` (TIMESTAMPTZ). Index on `(group_key_id, status)`.

#### Scenario: Table structure
- **WHEN** the migration runs
- **THEN** the `key_subscriptions` table SHALL be created with all specified columns, foreign keys, and index

### Requirement: Assign subscription to sub-key
The system SHALL allow assigning a plan to a sub-key via POST `/api/admin/groups/:group_id/keys/:key_id/subscriptions` with body `{ "plan_id": "<uuid>" }`. The system SHALL snapshot all plan values into the subscription record.

#### Scenario: Assign plan to sub-key
- **WHEN** an admin sends POST with `{ "plan_id": "<uuid>" }` referencing an active plan
- **THEN** the system SHALL create a `key_subscriptions` record with `sub_type`, `cost_limit_usd`, `model_limits`, `reset_hours`, `duration_days` copied from the plan, `status: "active"`, `activated_at: NULL`, and return it with status 201

#### Scenario: Assign inactive plan
- **WHEN** an admin sends POST referencing a plan with `is_active: false`
- **THEN** the system SHALL return 400 with error "Plan is not active"

#### Scenario: Plan not found
- **WHEN** an admin sends POST with a non-existent plan_id
- **THEN** the system SHALL return 404

### Requirement: List subscriptions for a sub-key
The system SHALL return all subscriptions for a sub-key via GET `/api/admin/groups/:group_id/keys/:key_id/subscriptions`.

#### Scenario: List subscriptions
- **WHEN** an admin sends GET for a sub-key's subscriptions
- **THEN** the system SHALL return all subscriptions ordered by `created_at` descending

### Requirement: Cancel subscription
The system SHALL allow cancelling a subscription via PATCH `/api/admin/groups/:group_id/keys/:key_id/subscriptions/:sub_id` with `{ "status": "cancelled" }`. Only subscriptions with `status: "active"` can be cancelled.

#### Scenario: Cancel active subscription
- **WHEN** an admin sends PATCH with `{ "status": "cancelled" }` for an active subscription
- **THEN** the system SHALL set `status: "cancelled"` and invalidate the Redis subscription cache for this key

#### Scenario: Cancel non-active subscription
- **WHEN** an admin sends PATCH with `{ "status": "cancelled" }` for an exhausted or expired subscription
- **THEN** the system SHALL return 400 with error "Only active subscriptions can be cancelled"

### Requirement: Subscription status lifecycle
A subscription SHALL have one of four statuses: `active`, `exhausted`, `expired`, `cancelled`. Transitions: `active` → `exhausted` (total budget depleted), `active` → `expired` (time exceeded), `active` → `cancelled` (admin action). Terminal states (`exhausted`, `expired`, `cancelled`) are permanent.

#### Scenario: Status transitions
- **WHEN** a subscription is in status `active`
- **THEN** it MAY transition to `exhausted`, `expired`, or `cancelled`

#### Scenario: Terminal states
- **WHEN** a subscription is in status `exhausted`, `expired`, or `cancelled`
- **THEN** it SHALL NOT transition to any other status

### Requirement: Lazy activation
A subscription's `activated_at` SHALL be set on the first proxy request that charges it, not at creation time. `expires_at` SHALL be computed as `activated_at + duration_days`. The activation SHALL use Redis SETNX for race-safe first-request handling.

#### Scenario: First request activates subscription
- **WHEN** a proxy request charges a subscription with `activated_at: NULL`
- **THEN** the system SHALL set `activated_at` to the current timestamp and compute `expires_at`

#### Scenario: Concurrent first requests
- **WHEN** two proxy requests simultaneously attempt to activate the same subscription
- **THEN** only the first SHALL set `activated_at` via Redis SETNX, and the second SHALL read the stored value

### Requirement: Drop monthly limit columns
The system SHALL remove `monthly_token_limit` and `monthly_request_limit` columns from the `group_keys` table.

#### Scenario: Migration removes columns
- **WHEN** the migration runs
- **THEN** the `group_keys` table SHALL no longer have `monthly_token_limit` or `monthly_request_limit` columns
