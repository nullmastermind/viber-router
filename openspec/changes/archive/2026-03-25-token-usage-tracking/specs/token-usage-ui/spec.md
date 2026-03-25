## ADDED Requirements

### Requirement: Token usage section in Group Detail page
The Group Detail page SHALL display a "Token Usage" section showing aggregated token usage statistics per server within the group, with filtering controls.

#### Scenario: Display token usage data
- **WHEN** an admin views the Group Detail page for a group that has token usage data in the selected time range
- **THEN** the page SHALL show a table with columns: server name, model, total input tokens, total output tokens, total cache creation tokens, total cache read tokens, and request count

#### Scenario: Date range filter
- **WHEN** the admin selects a date range using the date range picker
- **THEN** the section SHALL reload data for the selected range

#### Scenario: Server filter
- **WHEN** the admin selects a specific server from the server filter dropdown
- **THEN** the section SHALL display only usage data for that server

#### Scenario: Dynamic key filter
- **WHEN** the admin toggles the "Dynamic keys only" filter
- **THEN** the section SHALL display only usage records where `is_dynamic_key` is true

#### Scenario: Key hash filter
- **WHEN** the admin enters or selects a key hash value
- **THEN** the section SHALL display only usage records matching that key_hash

#### Scenario: Loading state
- **WHEN** the token usage data is being fetched
- **THEN** the section SHALL display a loading indicator

#### Scenario: Error state
- **WHEN** the token usage data fetch fails
- **THEN** the section SHALL display an error message with a retry option

#### Scenario: Empty state
- **WHEN** no token usage data exists for the selected filters
- **THEN** the section SHALL display a message indicating no data is available for the selected range
