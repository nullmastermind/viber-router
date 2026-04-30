## Requirements

### Requirement: Bonus subscription model allowlist storage
The system SHALL store an optional `bonus_allowed_models` list on bonus subscriptions. The list MUST contain model name strings. A `NULL` value or an empty list SHALL mean the bonus subscription accepts all request models.

#### Scenario: Create unrestricted bonus subscription
- **WHEN** an admin creates a bonus subscription without `bonus_allowed_models` or with an empty list
- **THEN** the created subscription SHALL be treated as accepting all request models

#### Scenario: Create restricted bonus subscription
- **WHEN** an admin creates a bonus subscription with `bonus_allowed_models = ["claude-sonnet-4-5", "claude-opus-4-1"]`
- **THEN** the created subscription SHALL store those model name strings on the bonus subscription record

### Requirement: Bonus subscription model allowlist enforcement
The subscription engine SHALL apply `bonus_allowed_models` when determining eligible bonus servers for a request model. If a bonus subscription has a non-empty allowlist, it SHALL be eligible only when the request model exactly matches one of the stored model names. If the allowlist is `NULL` or empty, the bonus subscription SHALL remain eligible for all request models.

#### Scenario: Restricted bonus matches request model
- **WHEN** `check_subscriptions()` evaluates an active bonus subscription with `bonus_allowed_models = ["claude-sonnet-4-5"]` for request model `claude-sonnet-4-5`
- **THEN** the bonus subscription SHALL be included in the returned bonus server list

#### Scenario: Restricted bonus does not match request model
- **WHEN** `check_subscriptions()` evaluates an active bonus subscription with `bonus_allowed_models = ["claude-sonnet-4-5"]` for request model `claude-opus-4-1`
- **THEN** the bonus subscription SHALL NOT be included in the returned bonus server list

#### Scenario: Unrestricted bonus accepts any request model
- **WHEN** `check_subscriptions()` evaluates an active bonus subscription with `bonus_allowed_models = NULL` or `[]`
- **THEN** the bonus subscription SHALL be included in the returned bonus server list regardless of request model

### Requirement: Bonus allowed models admin UI
The admin group detail UI SHALL allow admins to select zero or more model names when creating a bonus subscription. The options SHALL come from the group's loaded allowed models. The UI SHALL submit selected model names, not model IDs. Bonus subscription rows SHALL display the configured allowed model names, or "All models" when the allowlist is `NULL` or empty.

#### Scenario: Add Bonus dialog shows group model options
- **WHEN** the admin opens the Add Bonus dialog for a group with allowed models loaded
- **THEN** the dialog SHALL include a multi-select field whose options are the group's allowed model names

#### Scenario: Submit bonus selected model names
- **WHEN** the admin selects model options and submits the Add Bonus dialog
- **THEN** the request body SHALL include `bonus_allowed_models` as an array of selected model name strings

#### Scenario: Display unrestricted bonus row
- **WHEN** a bonus subscription row has `bonus_allowed_models = NULL` or `[]`
- **THEN** the row SHALL display "All models" for allowed models

#### Scenario: Display restricted bonus row
- **WHEN** a bonus subscription row has `bonus_allowed_models = ["claude-sonnet-4-5", "claude-opus-4-1"]`
- **THEN** the row SHALL display those model names in the bonus subscription row
