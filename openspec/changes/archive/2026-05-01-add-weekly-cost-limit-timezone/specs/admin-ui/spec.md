## ADDED Requirements

### Requirement: Settings timezone selector
The admin Settings page SHALL include a General card with a timezone select input backed by the global settings API. The default selected timezone SHALL be `Asia/Ho_Chi_Minh`.

#### Scenario: Timezone displayed in settings
- **WHEN** an admin opens the Settings page
- **THEN** the page SHALL show a General card containing a timezone select with the current `settings.timezone` value

#### Scenario: Timezone default
- **WHEN** the backend settings response has no explicit timezone value
- **THEN** the Settings page SHALL display `Asia/Ho_Chi_Minh` as the selected timezone

#### Scenario: Save timezone
- **WHEN** an admin selects a timezone and saves settings
- **THEN** the frontend SHALL send the selected `timezone` in the PUT `/api/admin/settings` request

### Requirement: Group detail weekly subscription limit display
The admin group detail page SHALL display each key subscription's weekly cost limit in the subscription table.

#### Scenario: Weekly limit shown for assigned subscription
- **WHEN** a key subscription has `weekly_cost_limit_usd: 30.0`
- **THEN** the subscription table SHALL display `$30.00` for that subscription's weekly cost limit

#### Scenario: Unlimited weekly limit shown
- **WHEN** a key subscription has `weekly_cost_limit_usd: null`
- **THEN** the subscription table SHALL display an unlimited or empty weekly limit value
