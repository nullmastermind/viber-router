## Purpose
TBD

## Requirements
### Requirement: Model pricing fields
The `models` table SHALL have 4 nullable FLOAT8 columns: `input_1m_usd`, `output_1m_usd`, `cache_write_1m_usd`, `cache_read_1m_usd`, representing the cost in USD per 1 million tokens for each token type.

#### Scenario: Model with no pricing configured
- **WHEN** a model record has all 4 pricing fields set to NULL
- **THEN** the system SHALL treat the model as having no pricing configured

#### Scenario: Model with partial pricing
- **WHEN** a model record has some pricing fields set and others NULL
- **THEN** the system SHALL use the set values for cost calculation and treat NULL fields as not configured (cost contribution = NULL for that token type)

#### Scenario: Model with zero pricing
- **WHEN** a model record has a pricing field set to 0
- **THEN** the system SHALL treat that token type as free (cost = $0.00)

### Requirement: Update model pricing via API
The system SHALL provide a `PUT /api/admin/models/:id` endpoint that accepts pricing fields to update a model's pricing configuration.

#### Scenario: Set all pricing fields
- **WHEN** an admin sends `PUT /api/admin/models/:id` with `{ "input_1m_usd": 3.0, "output_1m_usd": 15.0, "cache_write_1m_usd": 3.75, "cache_read_1m_usd": 0.30 }`
- **THEN** the system SHALL update the model's pricing fields and return the updated model record

#### Scenario: Clear pricing fields
- **WHEN** an admin sends `PUT /api/admin/models/:id` with `{ "input_1m_usd": null }`
- **THEN** the system SHALL set `input_1m_usd` to NULL (no pricing for that type)

#### Scenario: Reject negative pricing
- **WHEN** an admin sends a pricing field with a negative value
- **THEN** the system SHALL return HTTP 400 with an error message

#### Scenario: Update model name alongside pricing
- **WHEN** an admin sends `PUT /api/admin/models/:id` with `{ "name": "new-name", "input_1m_usd": 3.0 }`
- **THEN** the system SHALL update both the name and pricing fields

#### Scenario: Model not found
- **WHEN** an admin sends `PUT /api/admin/models/:id` with a non-existent ID
- **THEN** the system SHALL return HTTP 404

### Requirement: Models admin page
The system SHALL provide a dedicated Models page at route `/models` accessible from the sidebar navigation.

#### Scenario: List models with pricing
- **WHEN** an admin navigates to the Models page
- **THEN** the page SHALL display a table with columns: name, input ($/1MTok), output ($/1MTok), cache write ($/1MTok), cache read ($/1MTok), and actions

#### Scenario: Edit model pricing
- **WHEN** an admin clicks the edit button on a model row
- **THEN** the page SHALL open a dialog with fields for name and 4 pricing inputs (nullable, minimum 0)

#### Scenario: Create model with pricing
- **WHEN** an admin creates a new model
- **THEN** the create dialog SHALL include the 4 pricing fields (optional)

#### Scenario: Loading state
- **WHEN** the models data is being fetched
- **THEN** the page SHALL display a loading indicator

#### Scenario: Error state
- **WHEN** the models data fetch fails
- **THEN** the page SHALL display an error message with a retry option

#### Scenario: Empty state
- **WHEN** no models exist
- **THEN** the page SHALL display a message indicating no models are configured

#### Scenario: Display unset pricing
- **WHEN** a model has a NULL pricing field
- **THEN** the table SHALL display "—" for that field
