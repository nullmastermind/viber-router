## ADDED Requirements

### Requirement: Server cost rate multiplier fields
The `group_servers` table SHALL have 4 nullable FLOAT8 columns: `rate_input`, `rate_output`, `rate_cache_write`, `rate_cache_read`. When NULL, the effective rate SHALL be 1.0.

#### Scenario: Default rate
- **WHEN** a group_server record has all rate fields set to NULL
- **THEN** the system SHALL treat all rates as 1.0 (no markup or discount)

#### Scenario: Custom rate
- **WHEN** a group_server record has `rate_input` set to 1.5
- **THEN** the system SHALL multiply the input token cost by 1.5

#### Scenario: Zero rate
- **WHEN** a group_server record has a rate field set to 0
- **THEN** the system SHALL treat that token type as free on this server (cost multiplied by 0)

### Requirement: Update server cost rates via API
The system SHALL accept rate fields in the existing `PUT /api/admin/groups/:id/servers/:sid` endpoint.

#### Scenario: Set rate fields
- **WHEN** an admin sends `PUT /api/admin/groups/:id/servers/:sid` with `{ "rate_input": 1.5, "rate_output": 2.0 }`
- **THEN** the system SHALL update the specified rate fields and return the updated assignment

#### Scenario: Clear rate fields
- **WHEN** an admin sends a rate field with value null
- **THEN** the system SHALL set that rate to NULL (effective 1.0)

#### Scenario: Reject negative rates
- **WHEN** an admin sends a rate field with a negative value
- **THEN** the system SHALL return HTTP 400 with an error message

### Requirement: Rate tag display on server row
Each server row in the group detail servers tab SHALL display a clickable rate badge before the enable/disable toggle.

#### Scenario: All rates default
- **WHEN** all 4 rate fields are NULL or 1.0
- **THEN** the badge SHALL display "x1.0"

#### Scenario: Non-default rates
- **WHEN** any rate field differs from 1.0
- **THEN** the badge SHALL display the rate information (e.g., "x1.5" if input rate is 1.5)

#### Scenario: Click rate badge
- **WHEN** an admin clicks the rate badge
- **THEN** the system SHALL open a modal with 4 rate input fields (nullable, minimum 0, placeholder "1.0")

#### Scenario: Save rates from modal
- **WHEN** an admin edits rates in the modal and clicks save
- **THEN** the system SHALL call the update assignment API with the new rate values and update the badge display
