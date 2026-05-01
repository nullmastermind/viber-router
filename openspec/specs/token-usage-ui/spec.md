## Purpose
TBD

## Requirements
### Requirement: Token usage section in Group Detail page
The Group Detail page SHALL display a "Token Usage" section with two child tabs: "By Server" (showing aggregated token usage statistics per server) and "By Sub-Key" (showing aggregated usage per sub-key). The "By Server" tab SHALL be the default active child tab and SHALL contain all existing filtering controls and cost information unchanged.

#### Scenario: Display token usage data
- **WHEN** an admin views the Group Detail page for a group that has token usage data in the selected time range
- **THEN** the page SHALL show the "Token Usage" tab containing two child tabs: "By Server" and "By Sub-Key"

#### Scenario: Default child tab
- **WHEN** the admin navigates to the Token Usage tab
- **THEN** the "By Server" child tab SHALL be active by default, displaying the existing server/model usage table with all current filters (period toggle, server filter, dynamic key toggle, key hash filter)

#### Scenario: By Server tab content unchanged
- **WHEN** the admin views the "By Server" child tab
- **THEN** the tab SHALL display all existing content: period button toggle, server filter dropdown, dynamic key toggle, key hash filter, usage table with server/model columns, and total row -- with no changes to behavior or layout

#### Scenario: Cost column display with pricing
- **WHEN** a usage row has a model with pricing configured
- **THEN** the cost column SHALL display the calculated cost formatted as USD (e.g., "$1.23")

#### Scenario: Cost column display without pricing
- **WHEN** a usage row has a model with no pricing configured (NULL or no match)
- **THEN** the cost column SHALL display an em dash

#### Scenario: Total cost row
- **WHEN** the token usage table has data rows
- **THEN** the table SHALL display a total row at the bottom showing the sum of all cost values (excluding rows with no pricing)

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
