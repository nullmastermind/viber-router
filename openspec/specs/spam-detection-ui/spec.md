## Purpose
TBD

## Requirements
### Requirement: Spam tab in group detail
The admin UI SHALL provide a "Spam" tab in the Group Detail page that displays spam detection results for the selected group.

#### Scenario: Tab visible in group detail
- **WHEN** an admin navigates to a group detail page
- **THEN** a "Spam" tab SHALL be visible after the "Token Usage" tab

#### Scenario: Spam data loaded on tab activation
- **WHEN** the admin clicks the "Spam" tab
- **THEN** the system SHALL fetch spam detection results for the current group and display them in a paginated table

### Requirement: Spam results table
The spam tab SHALL display a paginated table with server-side pagination showing flagged keys.

#### Scenario: Table columns displayed
- **WHEN** spam results are loaded
- **THEN** the table SHALL show columns: Key (full api_key with copy button), Key Name, Spam Type (badge), Request Count, Peak RPM, Detected At

#### Scenario: Spam type badge colors
- **WHEN** a result has `spam_type: "low_token"`
- **THEN** the Spam Type cell SHALL display an orange `q-badge` with label "Low Token"

#### Scenario: Duplicate request badge
- **WHEN** a result has `spam_type: "duplicate_request"`
- **THEN** the Spam Type cell SHALL display a red `q-badge` with label "Duplicate Request"

#### Scenario: Full API key with copy button
- **WHEN** spam results are displayed
- **THEN** each row SHALL show the full unmasked API key and a copy-to-clipboard button following the same pattern as the Keys tab

#### Scenario: Server-side pagination
- **WHEN** the admin navigates to a different page in the spam table
- **THEN** the system SHALL fetch the corresponding page from the server and update the table

#### Scenario: Empty state
- **WHEN** no spam is detected for the group
- **THEN** the table SHALL display an appropriate empty state message

### Requirement: Spam store function
The groups Pinia store SHALL expose a `fetchSpamDetection(groupId, params)` function that calls the spam detection API endpoint.

#### Scenario: Store function fetches spam data
- **WHEN** `fetchSpamDetection` is called with a group ID and pagination params
- **THEN** it SHALL call `GET /api/admin/spam-detection?group_id=<id>&page=<p>&limit=<l>` and return the paginated response
