## MODIFIED Requirements

### Requirement: Plans list view
The Plans page SHALL display all subscription plans in a table with columns: Name, Type (fixed/hourly_reset/pay_per_request), Cost Limit, Weekly Cost Limit, RPM Limit, TPM Limit, Model Limits, Model Request Costs, Reset Hours, Duration (days), Active status.

#### Scenario: Display plans
- **WHEN** the user navigates to the Plans page
- **THEN** the system SHALL fetch and display all plans from GET `/api/admin/subscription-plans`, including each plan's `weekly_cost_limit_usd` and `tpm_limit`

#### Scenario: Empty state
- **WHEN** no plans exist
- **THEN** the system SHALL display "No subscription plans created"

#### Scenario: Weekly cost limit displayed
- **WHEN** a plan has `weekly_cost_limit_usd: 25.0`
- **THEN** the table SHALL display `$25.00` in the Weekly Cost Limit column

#### Scenario: Unlimited weekly cost displayed
- **WHEN** a plan has `weekly_cost_limit_usd: null`
- **THEN** the table SHALL display an unlimited or empty weekly limit value

#### Scenario: Model request costs displayed
- **WHEN** a plan has `sub_type: "pay_per_request"` and `model_request_costs: {"claude-sonnet-4-6": 0.10}`
- **THEN** the table SHALL display the model request costs as chips (e.g., "claude-sonnet-4-6: $0.10")

#### Scenario: TPM limit displayed
- **WHEN** a plan has `tpm_limit: 100000`
- **THEN** the table SHALL display `100000` in the TPM Limit column

### Requirement: Create plan dialog
The system SHALL provide a dialog to create a new plan with fields: name (text), type (select: fixed/hourly_reset/pay_per_request), cost limit (number), weekly cost limit (number, optional), RPM limit (number, optional), TPM limit (number, optional), duration days (number), reset hours (number, shown when type is hourly_reset OR pay_per_request), model limits editor (shown for all types), and model request costs editor (shown only when type is pay_per_request).

#### Scenario: Create fixed plan
- **WHEN** the user fills in the form with type "fixed" and submits
- **THEN** the system SHALL POST to `/api/admin/subscription-plans` and add the new plan to the table

#### Scenario: Create plan with weekly cost limit
- **WHEN** the user enters `25` in the Weekly Cost Limit field and submits
- **THEN** the request payload SHALL include `weekly_cost_limit_usd: 25`

#### Scenario: Create plan without weekly cost limit
- **WHEN** the user leaves the Weekly Cost Limit field empty and submits
- **THEN** the request payload SHALL include `weekly_cost_limit_usd: null` or omit it so the backend stores NULL

#### Scenario: Create plan with TPM limit
- **WHEN** the user enters `100000` in the TPM Limit field and submits
- **THEN** the request payload SHALL include `tpm_limit: 100000`

#### Scenario: Create hourly_reset plan
- **WHEN** the user selects type "hourly_reset"
- **THEN** the reset hours field SHALL become visible and required

#### Scenario: Create pay_per_request plan
- **WHEN** the user selects type "pay_per_request"
- **THEN** the model request costs editor SHALL become visible and the reset hours field SHALL become visible but optional

#### Scenario: pay_per_request requires model_request_costs
- **WHEN** the user selects type "pay_per_request" and submits without adding any model request costs
- **THEN** the system SHALL show a validation error and not submit

## ADDED Requirements

### Requirement: Edit plan weekly cost limit
The Plans page SHALL populate and submit `weekly_cost_limit_usd` when editing a plan.

#### Scenario: Open edit dialog with weekly limit
- **WHEN** the user edits a plan with `weekly_cost_limit_usd: 25.0`
- **THEN** the Weekly Cost Limit field SHALL be prefilled with `25.0`

#### Scenario: Clear weekly limit in edit dialog
- **WHEN** the user clears the Weekly Cost Limit field and saves
- **THEN** the request payload SHALL set `weekly_cost_limit_usd` to null
