## MODIFIED Requirements

### Requirement: Subscription cards display
The page SHALL display subscription information as cards, with active subscriptions first and inactive ones (expired/cancelled/exhausted) visually dimmed. Non-bonus subscription cards SHALL show lifetime cost progress and, when `weekly_cost_limit_usd` is present, weekly cost usage, weekly limit, and weekly reset time. Bonus subscriptions SHALL be rendered with a distinct card style separate from budget-based subscriptions. The page SHALL render a Custom Endpoints section after subscription cards and before the token usage table.

#### Scenario: Active subscription card
- **WHEN** a subscription has status `active` and `sub_type` is not `bonus`
- **THEN** the card shows: subscription type, cost_used / cost_limit_usd (as progress), weekly_cost_used / weekly_cost_limit_usd when present, weekly_reset_at when present, status badge, and expires_at (if set)

#### Scenario: Weekly limit hidden when unlimited
- **WHEN** a non-bonus subscription has `weekly_cost_limit_usd: null`
- **THEN** the card SHALL NOT display a weekly limit progress indicator

#### Scenario: Hourly reset subscription shows countdown
- **WHEN** a subscription has type `hourly_reset` and `window_reset_at` is set
- **THEN** the card displays the remaining time until quota reset (e.g., "Resets in 2h 15m")

#### Scenario: Weekly reset shown
- **WHEN** a non-bonus subscription has `weekly_reset_at` set
- **THEN** the card displays the remaining time or formatted time until the weekly cost limit resets

#### Scenario: Inactive subscription card
- **WHEN** a subscription has status `expired`, `cancelled`, or `exhausted`
- **THEN** the card is visually dimmed (reduced opacity) and shows the status badge

#### Scenario: No subscriptions
- **WHEN** the sub-key has no subscriptions
- **THEN** the page shows "No subscriptions" in the subscriptions section

#### Scenario: Bonus subscription card — with quota data
- **WHEN** a subscription has `sub_type = 'bonus'` and `bonus_quotas` is a non-empty array
- **THEN** the card shows `bonus_name` as title with a lightning bolt icon, one q-linear-progress bar per quota entry with utilization %, quota name label, and a reset countdown if `reset_at` is present, and a list of `bonus_usage` model/count pairs

#### Scenario: Bonus subscription card — empty quotas (fetch failed)
- **WHEN** a subscription has `sub_type = 'bonus'` and `bonus_quotas` is an empty array
- **THEN** the card shows "Quota info unavailable" in the quota section but still renders `bonus_usage`

#### Scenario: Bonus subscription card — null quotas (no URL configured)
- **WHEN** a subscription has `sub_type = 'bonus'` and `bonus_quotas` is null
- **THEN** the quota section of the bonus card is not rendered

#### Scenario: Custom Endpoints section placement
- **WHEN** a valid sub-key's usage data is displayed
- **THEN** the Custom Endpoints section appears between the subscriptions section and token usage table

## ADDED Requirements

### Requirement: Custom endpoint cards
The public usage page SHALL display user endpoints as cards in a Custom Endpoints section. Each card SHALL show the endpoint name, truncated base URL, priority mode badge, enabled toggle, edit button, delete button, quota display when quota data is configured, and 30-day usage statistics.

#### Scenario: Endpoint card rendered
- **WHEN** the public usage response includes a user endpoint
- **THEN** the Custom Endpoints section shows a card with its name, truncated base URL, priority mode, enabled state, edit/delete controls, and usage stats

#### Scenario: Quota configured
- **WHEN** a user endpoint includes quota data
- **THEN** the card displays quota information using the same visual pattern as bonus quota display

#### Scenario: Quota not configured
- **WHEN** a user endpoint has no quota URL or quota data
- **THEN** the card omits the quota display without hiding usage stats or management controls

#### Scenario: No custom endpoints
- **WHEN** the valid sub-key has no user endpoints
- **THEN** the Custom Endpoints section shows an empty state and an Add Endpoint action

### Requirement: Custom endpoint add and edit dialog
The public usage page SHALL provide persistent add and edit dialogs for user endpoints following existing Quasar form patterns. The dialog SHALL include fields for Name, Base URL, API Key, Model Mappings JSON, Priority Mode, Quota URL, and Quota Headers JSON. Name, Base URL, and API Key SHALL be required when creating an endpoint.

#### Scenario: Add dialog opens
- **WHEN** the user clicks Add Endpoint
- **THEN** a persistent dialog opens with empty fields for all endpoint settings

#### Scenario: Edit dialog opens
- **WHEN** the user clicks edit on an endpoint card
- **THEN** a persistent dialog opens with that endpoint's existing fields pre-populated

#### Scenario: Save create success
- **WHEN** the user submits valid fields in the add dialog and the API succeeds
- **THEN** the page shows a success notification, closes the dialog, and refreshes the endpoint list

#### Scenario: Save update success
- **WHEN** the user submits valid changes in the edit dialog and the API succeeds
- **THEN** the page shows a success notification, closes the dialog, and refreshes the endpoint list

#### Scenario: Invalid JSON in dialog
- **WHEN** the Model Mappings or Quota Headers textarea contains invalid JSON
- **THEN** the page shows a clear validation error and SHALL NOT submit the request

#### Scenario: Save API error
- **WHEN** create or update fails
- **THEN** the page keeps the dialog open, clears loading state, and shows an error notification

### Requirement: Custom endpoint enable toggle and delete
The public usage page SHALL allow users to enable or disable an endpoint and delete an endpoint. Delete actions SHALL require confirmation before calling the API.

#### Scenario: Toggle endpoint enabled state
- **WHEN** the user changes an endpoint enabled toggle
- **THEN** the page sends a PATCH request for `is_enabled`, shows success or error feedback, and refreshes endpoint state

#### Scenario: Delete confirmation accepted
- **WHEN** the user confirms deletion of an endpoint
- **THEN** the page sends a DELETE request, shows success feedback, and removes the endpoint from the displayed list

#### Scenario: Delete confirmation cancelled
- **WHEN** the user cancels the delete confirmation
- **THEN** the page SHALL NOT call the delete API and the endpoint remains displayed

### Requirement: Custom endpoint max limit UI
The public usage page SHALL enforce and clearly communicate the maximum of 10 user endpoints per sub-key.

#### Scenario: Under limit add enabled
- **WHEN** the user has fewer than 10 custom endpoints
- **THEN** the Add Endpoint action is available

#### Scenario: At limit add blocked
- **WHEN** the user already has 10 custom endpoints
- **THEN** the page disables or blocks adding another endpoint and displays a clear max-limit message

#### Scenario: Server max-limit response
- **WHEN** the create API rejects an endpoint because the sub-key already has 10 endpoints
- **THEN** the page shows the max-limit error message to the user
