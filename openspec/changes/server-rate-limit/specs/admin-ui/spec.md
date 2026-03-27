## MODIFIED Requirements

### Requirement: Group detail page
The admin UI SHALL display a group's full configuration including its servers with priorities and model mappings. The page SHALL support editing group properties, managing server assignments, reordering priorities, and toggling server enabled status. **Each server in the list SHALL display a rate limit badge showing `{max_requests}/{rate_window_seconds}s` when rate limiting is configured. The Edit Server dialog SHALL include a Rate Limit section below the Circuit Breaker section with two number inputs: "Max Requests" and "Window (seconds)".**

#### Scenario: View group detail
- **WHEN** the user clicks on a group row in the groups table
- **THEN** the group detail page shows: name, API key (with copy and regenerate buttons), status toggle, failover status codes editor, and a list of assigned servers ordered by priority

#### Scenario: Edit group properties
- **WHEN** the user modifies the group name or failover codes and clicks Save
- **THEN** the group is updated

#### Scenario: Copy API key
- **WHEN** the user clicks the Copy button next to the API key
- **THEN** the API key is copied to the clipboard

#### Scenario: Regenerate API key
- **WHEN** the user clicks Regenerate and confirms
- **THEN** a new API key is generated and displayed

#### Scenario: Add server to group
- **WHEN** the user clicks "Add Server", selects a server from a dropdown, sets priority, and optionally adds model mappings
- **THEN** the server is added to the group's server list

#### Scenario: Edit model mappings
- **WHEN** the user clicks on a server's model mappings section and adds/edits/removes mapping entries
- **THEN** the model mappings are updated

#### Scenario: Reorder servers by drag
- **WHEN** the user drags a server row to a new position in the server list
- **THEN** the server priorities are updated to reflect the new order

#### Scenario: Remove server from group
- **WHEN** the user clicks Remove on a server in the group's server list and confirms
- **THEN** the server is removed from the group

#### Scenario: Toggle server enabled status
- **WHEN** the user clicks the toggle switch on a server row in the group detail page
- **THEN** the server's `is_enabled` status SHALL be toggled immediately via PUT API call without a confirmation dialog, and the server list SHALL reload

#### Scenario: Disabled server visual state
- **WHEN** a server in the group has `is_enabled: false`
- **THEN** the server row SHALL appear dimmed (reduced opacity) with the server name displayed in strikethrough, and the toggle switch SHALL be in the OFF position

#### Scenario: Enabled server visual state
- **WHEN** a server in the group has `is_enabled: true`
- **THEN** the server row SHALL appear at full opacity with normal text, and the toggle switch SHALL be in the ON position

#### Scenario: Rate limit badge displayed
- **WHEN** a server has `max_requests=100` and `rate_window_seconds=60`
- **THEN** the server row SHALL display a badge showing "100/60s"

#### Scenario: No rate limit badge when not configured
- **WHEN** a server has `max_requests=NULL` and `rate_window_seconds=NULL`
- **THEN** the server row SHALL NOT display a rate limit badge

#### Scenario: Edit rate limit in Edit Server dialog
- **WHEN** the user clicks Edit on a server and the Edit Server dialog opens
- **THEN** the dialog SHALL include a "Rate Limit" section below the Circuit Breaker section with "Max Requests" and "Window (seconds)" number inputs, pre-filled with current values or empty if not configured

#### Scenario: Save rate limit configuration
- **WHEN** the user enters `max_requests=100` and `rate_window_seconds=60` in the Edit Server dialog and clicks Save
- **THEN** the rate limit fields SHALL be saved via PUT API call and the server list SHALL reload

#### Scenario: Clear rate limit configuration
- **WHEN** the user clears both rate limit fields in the Edit Server dialog and clicks Save
- **THEN** both fields SHALL be sent as null via PUT API call
