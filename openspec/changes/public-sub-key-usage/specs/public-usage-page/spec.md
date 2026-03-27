## ADDED Requirements

### Requirement: Public usage page routes
The system SHALL provide a public page at `/usage` (with key input form) and `/usage/:key` (direct link). Both routes MUST be accessible without admin login.

#### Scenario: Access via form
- **WHEN** a user navigates to `/#/usage`
- **THEN** the page displays a form with a text input for the sub-key and a submit button

#### Scenario: Access via direct link
- **WHEN** a user navigates to `/#/usage/sk-vibervn-abc123`
- **THEN** the page automatically loads and displays usage data for that sub-key

#### Scenario: Form submission navigates to key route
- **WHEN** a user enters a sub-key in the form and submits
- **THEN** the browser navigates to `/#/usage/<entered-key>` and displays the usage data

### Requirement: Page is outside admin layout
The page SHALL render outside `MainLayout.vue` (no sidebar, no admin navigation). It follows the same pattern as `LoginPage.vue`.

#### Scenario: No admin UI elements
- **WHEN** the public usage page is displayed
- **THEN** there is no left drawer, no admin navigation links, no admin header

### Requirement: Navigation guard exempts usage paths
The Vue Router `beforeEach` guard SHALL allow navigation to paths starting with `/usage` without an admin token in localStorage.

#### Scenario: Unauthenticated access to usage page
- **WHEN** a user with no admin token navigates to `/#/usage/sk-vibervn-abc123`
- **THEN** the page loads normally without redirecting to `/login`

### Requirement: Key info display
When a valid key is loaded, the page SHALL display the key name and group name.

#### Scenario: Key info shown
- **WHEN** the API returns data for a valid sub-key
- **THEN** the page displays the key name and group name prominently at the top

### Requirement: Subscription cards display
The page SHALL display subscription information as cards, with active subscriptions first and inactive ones (expired/cancelled/exhausted) visually dimmed.

#### Scenario: Active subscription card
- **WHEN** a subscription has status `active`
- **THEN** the card shows: subscription type, cost_used / cost_limit_usd (as progress), status badge, and expires_at (if set)

#### Scenario: Hourly reset subscription shows countdown
- **WHEN** a subscription has type `hourly_reset` and `window_reset_at` is set
- **THEN** the card displays the remaining time until quota reset (e.g., "Resets in 2h 15m")

#### Scenario: Inactive subscription card
- **WHEN** a subscription has status `expired`, `cancelled`, or `exhausted`
- **THEN** the card is visually dimmed (reduced opacity) and shows the status badge

#### Scenario: No subscriptions
- **WHEN** the sub-key has no subscriptions
- **THEN** the page shows "No subscriptions" in the subscriptions section

### Requirement: Token usage table display
The page SHALL display a table of token usage aggregated by model for the last 30 days.

#### Scenario: Usage table with data
- **WHEN** the sub-key has usage data
- **THEN** the table shows columns: Model, Input Tokens, Output Tokens, Cache Creation, Cache Read, Requests, Cost ($)

#### Scenario: No usage data
- **WHEN** the sub-key has no usage data in the last 30 days
- **THEN** the page shows "No usage data" in the usage section

### Requirement: Error handling
The page SHALL display appropriate error messages for API errors.

#### Scenario: Invalid key error
- **WHEN** the API returns 403 (invalid or inactive key)
- **THEN** the page displays "Invalid or inactive key" error message

#### Scenario: Rate limit error
- **WHEN** the API returns 429 (too many requests)
- **THEN** the page displays "Too many requests. Please try again later." error message

### Requirement: Loading state
The page SHALL show a loading indicator while fetching data from the API.

#### Scenario: Loading indicator
- **WHEN** the page is fetching usage data
- **THEN** a loading spinner is displayed
- **WHEN** the data is loaded or an error occurs
- **THEN** the loading spinner is replaced with the data or error message
