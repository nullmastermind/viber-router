## ADDED Requirements

### Requirement: Subscriptions section in expanded sub-key row
The expanded sub-key row in GroupDetailPage Keys tab SHALL display a "Subscriptions" section positioned before the SubKeyUsage component. It SHALL show a table of all subscriptions for that key.

#### Scenario: Display subscriptions
- **WHEN** the user expands a sub-key row
- **THEN** the system SHALL fetch subscriptions from GET `/api/admin/groups/:id/keys/:key_id/subscriptions` and display them in a table with columns: Plan (name from snapshot or plan reference or bonus_name for bonus subs), Type, Budget, Used, Status

#### Scenario: No subscriptions
- **WHEN** a sub-key has no subscriptions
- **THEN** the system SHALL display "No subscriptions — unlimited usage"

#### Scenario: Bonus subscription row display
- **WHEN** a subscription row has `sub_type = 'bonus'`
- **THEN** the Plan column shows `bonus_name`, the Type column shows "Bonus", and Budget/Used columns show "N/A"

### Requirement: Add subscription button
The subscriptions section SHALL include an "Add Subscription" button that opens a dropdown of active plans.

#### Scenario: Add subscription
- **WHEN** the admin clicks "Add Subscription" and selects a plan from the dropdown
- **THEN** the system SHALL POST to `/api/admin/groups/:id/keys/:key_id/subscriptions` with `{ "plan_id": "<uuid>" }` and add the new subscription to the table

#### Scenario: No active plans
- **WHEN** there are no active plans
- **THEN** the dropdown SHALL be empty with a message "No active plans available"

### Requirement: Add Bonus button
The subscriptions section SHALL include an "Add Bonus" button alongside the "Add Subscription" button. Clicking "Add Bonus" SHALL open a dialog with: Name (text input, required), Base URL (text input, required, prefilled with "https://api.anthropic.com"), API Key (text input, required), Quota Check URL (text input, optional), Quota Headers (text input for JSON, optional). Submitting the dialog SHALL POST the bonus fields to the subscriptions endpoint and add the new row to the table.

#### Scenario: Open Add Bonus dialog
- **WHEN** the admin clicks the "Add Bonus" button
- **THEN** a dialog opens with Name, Base URL (prefilled "https://api.anthropic.com"), API Key, Quota Check URL, and Quota Headers fields

#### Scenario: Submit Add Bonus dialog — success
- **WHEN** the admin fills the required fields (Name, Base URL, API Key) and submits
- **THEN** the system POSTs `{ "bonus_name": "...", "bonus_base_url": "...", "bonus_api_key": "...", ... }` to the subscriptions endpoint and adds the new row to the table

#### Scenario: Submit Add Bonus dialog — missing required field
- **WHEN** the admin tries to submit the dialog without filling a required field
- **THEN** the system SHALL show a validation error and not submit

### Requirement: Cancel subscription action
Each active subscription row SHALL have a "Cancel" action button.

#### Scenario: Cancel subscription
- **WHEN** the admin clicks "Cancel" on an active subscription
- **THEN** the system SHALL PATCH `/api/admin/groups/:id/keys/:key_id/subscriptions/:sub_id` with `{ "status": "cancelled" }` and update the row status

#### Scenario: Terminal subscription
- **WHEN** a subscription is in status exhausted, expired, or cancelled
- **THEN** the "Cancel" button SHALL NOT be shown

### Requirement: Subscription usage display
Each non-bonus subscription row SHALL display the current cost usage against the budget.

#### Scenario: Fixed subscription usage
- **WHEN** a fixed subscription has used $340 of $1000
- **THEN** the Used column SHALL display "$340.00 / $1000.00"

#### Scenario: Hourly subscription usage
- **WHEN** an hourly_reset subscription has used $45 of $100 in the current window
- **THEN** the Used column SHALL display "$45.00 / $100.00 (window)"

### Requirement: Subscription status badges
Each subscription status SHALL be displayed with a colored badge: active (green), exhausted (red), expired (orange), cancelled (grey).

#### Scenario: Status display
- **WHEN** a subscription has status "active"
- **THEN** the status SHALL be displayed as a green badge with text "active"

### Requirement: Subscription tables show TPM limit
The subscription tables in the group detail page SHALL display each key subscription's optional `tpm_limit` in a TPM Limit column.

#### Scenario: Subscription has TPM limit
- **WHEN** a key subscription has `tpm_limit: 100000`
- **THEN** the group detail subscription table SHALL display `100000` in the TPM Limit column for that subscription

#### Scenario: Subscription has no TPM limit
- **WHEN** a key subscription has `tpm_limit: null`
- **THEN** the group detail subscription table SHALL display an empty or unlimited value for the TPM Limit column

