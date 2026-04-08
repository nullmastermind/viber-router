## MODIFIED Requirements

### Requirement: Plans list view
The Plans page SHALL display all subscription plans in a table with columns: Name, Type (fixed/hourly_reset/pay_per_request), Cost Limit, Model Limits, Model Request Costs, Reset Hours, Duration (days), Active status.

#### Scenario: Display plans
- **WHEN** the user navigates to the Plans page
- **THEN** the system SHALL fetch and display all plans from GET `/api/admin/subscription-plans`

#### Scenario: Empty state
- **WHEN** no plans exist
- **THEN** the system SHALL display "No subscription plans created"

#### Scenario: Model request costs displayed
- **WHEN** a plan has `sub_type: "pay_per_request"` and `model_request_costs: {"claude-sonnet-4-6": 0.10}`
- **THEN** the table SHALL display the model request costs as chips (e.g., "claude-sonnet-4-6: $0.10")

### Requirement: Create plan dialog
The system SHALL provide a dialog to create a new plan with fields: name (text), type (select: fixed/hourly_reset/pay_per_request), cost limit (number), duration days (number), reset hours (number, shown when type is hourly_reset OR pay_per_request), model limits editor (shown for all types), and model request costs editor (shown only when type is pay_per_request).

#### Scenario: Create fixed plan
- **WHEN** the user fills in the form with type "fixed" and submits
- **THEN** the system SHALL POST to `/api/admin/subscription-plans` and add the new plan to the table

#### Scenario: Create hourly_reset plan
- **WHEN** the user selects type "hourly_reset"
- **THEN** the reset hours field SHALL become visible and required

#### Scenario: Create pay_per_request plan
- **WHEN** the user selects type "pay_per_request"
- **THEN** the model request costs editor SHALL become visible and the reset hours field SHALL become visible but optional

#### Scenario: pay_per_request requires model_request_costs
- **WHEN** the user selects type "pay_per_request" and submits without adding any model request costs
- **THEN** the system SHALL show a validation error and not submit

### Requirement: Model request costs editor
The create/edit plan dialog SHALL include a model request costs editor (visible only when `sub_type` is `pay_per_request`) where the admin can add per-model flat costs by selecting a model from a dropdown and entering a dollar amount per request.

#### Scenario: Add model request cost
- **WHEN** the admin selects "claude-sonnet-4-6" from the dropdown and enters 0.10
- **THEN** the model request cost SHALL be added to `model_request_costs` as `{"claude-sonnet-4-6": 0.10}`

#### Scenario: Remove model request cost
- **WHEN** the admin removes a model request cost entry
- **THEN** the model SHALL be removed from `model_request_costs`

#### Scenario: Editor hidden for non-pay_per_request types
- **WHEN** the plan type is "fixed" or "hourly_reset"
- **THEN** the model request costs editor SHALL NOT be visible
