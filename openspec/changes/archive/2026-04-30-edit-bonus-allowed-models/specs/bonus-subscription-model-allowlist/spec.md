## MODIFIED Requirements

### Requirement: Bonus allowed models admin UI
The admin group detail UI SHALL allow admins to select zero or more model names when creating a bonus subscription. The options SHALL come from the group's loaded allowed models. The UI SHALL submit selected model names, not model IDs. Bonus subscription rows SHALL display the configured allowed model names, or "All models" when the allowlist is `NULL` or empty. For active bonus subscription rows, the UI SHALL allow admins to edit the allowed model list after creation using the same group allowed model options. The UI MUST NOT allow editing allowed models for non-bonus subscriptions or inactive bonus subscriptions.

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

#### Scenario: Edit active bonus row allowed models
- **WHEN** an admin clicks the allowed models cell for an active bonus subscription row
- **THEN** the UI SHALL present a multi-select editor populated with the group's allowed model names and the subscription's current selected values

#### Scenario: Save edited bonus allowed models
- **WHEN** an admin saves edited allowed models for an active bonus subscription
- **THEN** the UI SHALL call the bonus allowed models update API with `bonus_allowed_models` as an array of selected model name strings
- **AND** the UI SHALL refresh the subscription list after a successful response

#### Scenario: Do not edit ineligible rows
- **WHEN** a subscription row is not bonus type or is not active
- **THEN** the UI SHALL NOT offer an allowed models edit action for that row

### Requirement: Bonus allowed models update API
The admin API SHALL provide a focused mutation for updating `bonus_allowed_models` on an existing key subscription. The mutation SHALL accept `bonus_allowed_models` as an array of model name strings. The API SHALL only update subscriptions that exist, belong to the requested key, have `sub_type = 'bonus'`, and have active status. Empty arrays and `NULL` values SHALL continue to mean the bonus subscription accepts all request models. Successful updates SHALL invalidate the Redis subscription cache for the affected key and return the updated subscription.

#### Scenario: Update active bonus subscription allowlist
- **WHEN** an admin updates `bonus_allowed_models` for an active bonus subscription that belongs to the requested key
- **THEN** the API SHALL persist the new model name list
- **AND** the API SHALL invalidate `key_subs:{key_id}` in Redis
- **AND** the API SHALL return the updated subscription record

#### Scenario: Update bonus subscription to unrestricted
- **WHEN** an admin updates an active bonus subscription with `bonus_allowed_models = []`
- **THEN** the subscription SHALL be treated as accepting all request models

#### Scenario: Reject update for subscription that does not belong to key
- **WHEN** an admin attempts to update a subscription id that does not belong to the requested key
- **THEN** the API SHALL reject the request and SHALL NOT update any subscription

#### Scenario: Reject update for non-bonus subscription
- **WHEN** an admin attempts to update `bonus_allowed_models` for a non-bonus subscription
- **THEN** the API SHALL reject the request and SHALL NOT update the subscription

#### Scenario: Reject update for inactive bonus subscription
- **WHEN** an admin attempts to update `bonus_allowed_models` for an inactive bonus subscription
- **THEN** the API SHALL reject the request and SHALL NOT update the subscription
