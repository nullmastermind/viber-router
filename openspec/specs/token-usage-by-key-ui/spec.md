## ADDED Requirements

### Requirement: Sub-key usage table in Group Detail page
The Group Detail page SHALL display a "By Sub-Key" child tab within the Token Usage tab showing aggregated token usage per sub-key.

#### Scenario: Display sub-key usage data
- **WHEN** an admin views the "By Sub-Key" child tab for a group that has token usage data in the selected date range
- **THEN** the tab SHALL show a table with columns: Key Name, Input Tokens, Output Tokens, Cache Write, Cache Read, Requests, Cost ($), Created At

#### Scenario: Default date range
- **WHEN** the admin first opens the "By Sub-Key" tab
- **THEN** the date range SHALL default to 30 days ago (start) through now (end)

#### Scenario: Date range filtering
- **WHEN** the admin changes the start or end date picker values
- **THEN** the table SHALL reload data for the newly selected date range

#### Scenario: Column sorting
- **WHEN** the admin clicks a sortable column header (Input Tokens, Output Tokens, Cache Write, Cache Read, Requests, Cost)
- **THEN** the table SHALL sort by that column, toggling between ascending and descending on repeated clicks

#### Scenario: Master/dynamic key row
- **WHEN** usage records exist with NULL `group_key_id`
- **THEN** the table SHALL display a row with Key Name shown as "Master / Dynamic Keys"

#### Scenario: Deleted key row
- **WHEN** usage records reference a `group_key_id` whose key no longer exists
- **THEN** the table SHALL display the row with Key Name shown as "Deleted Key"

#### Scenario: Expand row to show subscriptions
- **WHEN** the admin clicks a sub-key usage row (not the master/dynamic key row)
- **THEN** the row SHALL expand to show that key's subscriptions table, reusing the existing `loadKeySubscriptions` logic and `subColumns` definition

#### Scenario: Master key row not expandable
- **WHEN** the admin clicks the master/dynamic key row (where `group_key_id` is null)
- **THEN** the row SHALL NOT expand (no subscriptions to show)

#### Scenario: Total row
- **WHEN** the sub-key usage table has data rows
- **THEN** the table SHALL display a bold total row at the bottom summing all numeric columns (Input Tokens, Output Tokens, Cache Write, Cache Read, Requests, Cost)

#### Scenario: Loading state
- **WHEN** the sub-key usage data is being fetched
- **THEN** the tab SHALL display a centered spinner

#### Scenario: Error state
- **WHEN** the sub-key usage data fetch fails
- **THEN** the tab SHALL display an error banner with a "Retry" button

#### Scenario: Empty state
- **WHEN** no sub-key usage data exists for the selected date range
- **THEN** the tab SHALL display a banner with "No usage data for this period"

#### Scenario: Cost formatting
- **WHEN** a sub-key row has a non-null `cost_usd` value
- **THEN** the Cost column SHALL display the value formatted as `$X.XXXX` (4 decimal places)

#### Scenario: Cost null display
- **WHEN** a sub-key row has a null `cost_usd` value
- **THEN** the Cost column SHALL display an em dash
