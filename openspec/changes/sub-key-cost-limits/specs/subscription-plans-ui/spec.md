## ADDED Requirements

### Requirement: Plans sidebar page
The system SHALL display a "Plans" page accessible from the main sidebar navigation, at the same level as Groups and Models.

#### Scenario: Sidebar navigation
- **WHEN** the user views the sidebar
- **THEN** a "Plans" link SHALL be visible and navigate to the Plans page

### Requirement: Plans list view
The Plans page SHALL display all subscription plans in a table with columns: Name, Type (fixed/hourly_reset), Cost Limit, Model Limits, Reset Hours, Duration (days), Active status.

#### Scenario: Display plans
- **WHEN** the user navigates to the Plans page
- **THEN** the system SHALL fetch and display all plans from GET `/api/admin/subscription-plans`

#### Scenario: Empty state
- **WHEN** no plans exist
- **THEN** the system SHALL display "No subscription plans created"

### Requirement: Create plan dialog
The system SHALL provide a dialog to create a new plan with fields: name (text), type (select: fixed/hourly_reset), cost limit (number), duration days (number), reset hours (number, shown only when type is hourly_reset), and model limits editor.

#### Scenario: Create fixed plan
- **WHEN** the user fills in the form with type "fixed" and submits
- **THEN** the system SHALL POST to `/api/admin/subscription-plans` and add the new plan to the table

#### Scenario: Create hourly_reset plan
- **WHEN** the user selects type "hourly_reset"
- **THEN** the reset hours field SHALL become visible and required

### Requirement: Model limits editor
The create/edit plan dialog SHALL include a model limits editor where the admin can add per-model cost limits by selecting a model from a dropdown (populated from the `models` table) and entering a dollar amount.

#### Scenario: Add model limit
- **WHEN** the admin selects "claude-opus-4-6" from the dropdown and enters 30.0
- **THEN** the model limit SHALL be added to the `model_limits` JSONB as `{"claude-opus-4-6": 30.0}`

#### Scenario: Remove model limit
- **WHEN** the admin removes a model limit entry
- **THEN** the model SHALL be removed from the `model_limits` JSONB

### Requirement: Edit plan
The system SHALL allow editing a plan's fields via an edit action on each row.

#### Scenario: Edit plan
- **WHEN** the admin edits a plan and saves
- **THEN** the system SHALL PATCH `/api/admin/subscription-plans/:id` with the changed fields

### Requirement: Toggle plan active status
The system SHALL allow toggling a plan's `is_active` status directly from the table.

#### Scenario: Disable plan
- **WHEN** the admin toggles a plan to inactive
- **THEN** the system SHALL PATCH with `{ "is_active": false }` and the plan SHALL be visually distinguished as inactive
