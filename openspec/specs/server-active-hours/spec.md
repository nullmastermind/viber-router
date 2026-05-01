## Purpose
TBD

## Requirements
### Requirement: Admin UI displays active hours configuration on server cards
The group detail page server list SHALL display an active hours badge on each server card when `active_hours_start`, `active_hours_end`, and `active_hours_timezone` are all set. The badge SHALL show the window in the format `HH:MM-HH:MM (timezone)` (e.g., `08:00-23:00 (Asia/Ho_Chi_Minh)`). When the fields are all NULL, no active hours badge SHALL be shown.

#### Scenario: Server with active hours — badge displayed
- **WHEN** a server assignment has `active_hours_start="08:00"`, `active_hours_end="23:00"`, `active_hours_timezone="Asia/Ho_Chi_Minh"`
- **THEN** the server card SHALL display a badge reading `08:00-23:00 (Asia/Ho_Chi_Minh)`

#### Scenario: Server with no active hours — no badge
- **WHEN** a server assignment has all three active hours fields set to NULL
- **THEN** the server card SHALL NOT display any active hours badge

### Requirement: Admin UI edit server dialog has active hours section
The Edit Server Dialog in `GroupDetailPage.vue` SHALL include an "Active Hours" section with three fields: a timezone selector (`q-select`, filterable, with a curated list of common IANA timezones), a start time input (`q-input` with HH:MM mask), and an end time input (`q-input` with HH:MM mask). The section SHALL include hint text: "Leave empty for 24/7. Overnight windows supported (e.g., 22:00-06:00)." A "Clear" button SHALL reset all three fields to null/empty.

#### Scenario: Edit dialog opens — active hours fields pre-populated
- **WHEN** admin opens the Edit Server dialog for a server with active hours configured
- **THEN** the timezone selector SHALL show the configured timezone and start/end inputs SHALL show the configured times

#### Scenario: Edit dialog opens — active hours fields empty for 24/7 server
- **WHEN** admin opens the Edit Server dialog for a server with no active hours configured
- **THEN** all three active hours fields SHALL be empty

#### Scenario: Clear button resets all active hours fields
- **WHEN** admin clicks the "Clear" button in the Active Hours section
- **THEN** all three fields (timezone, start, end) SHALL be reset to empty/null

### Requirement: Admin UI enforces all-or-nothing validation for active hours
The Edit Server Dialog SHALL validate that either all three active hours fields are filled or all three are empty before allowing the save action. If only 1 or 2 fields are filled, the dialog SHALL display a validation error and disable the save button. The start time, end time, and timezone fields SHALL each be validated for correct format when non-empty (HH:MM for times; selection from the provided IANA list for timezone).

#### Scenario: All three fields filled — save allowed
- **WHEN** admin enters a timezone, start time, and end time
- **THEN** the save button SHALL be enabled (assuming other fields are valid)

#### Scenario: Only some fields filled — save blocked
- **WHEN** admin enters a start time and end time but leaves timezone empty
- **THEN** the dialog SHALL show a validation error and the save button SHALL remain disabled

#### Scenario: All three fields empty — save allowed (clears restriction)
- **WHEN** admin clears all three active hours fields
- **THEN** the save button SHALL be enabled and saving SHALL send null values for all three fields
