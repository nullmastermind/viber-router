## Purpose
TBD

## Requirements
### Requirement: Server cost rate multiplier fields
The `group_servers` table SHALL have 4 nullable FLOAT8 columns: `rate_input`, `rate_output`, `rate_cache_write`, `rate_cache_read`. When NULL, the effective rate SHALL be 1.0. The table SHALL also have `normalize_cache_read BOOLEAN NOT NULL DEFAULT false`.

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
The system SHALL accept rate fields and `normalize_cache_read` in the existing `PUT /api/admin/groups/:id/servers/:sid` endpoint.

#### Scenario: Set rate fields
- **WHEN** an admin sends `PUT /api/admin/groups/:id/servers/:sid` with `{ "rate_input": 1.5, "rate_output": 2.0 }`
- **THEN** the system SHALL update the specified rate fields and return the updated assignment

#### Scenario: Clear rate fields
- **WHEN** an admin sends a rate field with value null
- **THEN** the system SHALL set that rate to NULL (effective 1.0)

#### Scenario: Reject negative rates
- **WHEN** an admin sends a rate field with a negative value
- **THEN** the system SHALL return HTTP 400 with an error message

#### Scenario: Set normalize_cache_read
- **WHEN** an admin sends `PUT /api/admin/groups/:id/servers/:sid` with `{ "normalize_cache_read": true }`
- **THEN** the system SHALL update `normalize_cache_read` to `true` for that assignment

### Requirement: Cost calculation with normalize_cache_read
The `calculate_cost()` function SHALL accept a `normalize_cache_read: bool` parameter. When `true`, cache-read tokens SHALL be priced at `input_1m_usd × rate_input` instead of `cache_read_1m_usd × rate_cache_read`. Cache-creation tokens are always priced at `cache_write_1m_usd × rate_cache_write` regardless of the flag.

#### Scenario: normalize_cache_read false (default behavior)
- **WHEN** `normalize_cache_read` is `false` and a request has 100 cache-read tokens with `cache_read_1m_usd = 0.30` and `rate_cache_read = 1.0`
- **THEN** the cache-read cost SHALL be `100 × 0.30 × 1.0 / 1_000_000 = $0.00003`

#### Scenario: normalize_cache_read true
- **WHEN** `normalize_cache_read` is `true` and a request has 100 cache-read tokens with `input_1m_usd = 3.0` and `rate_input = 1.0`
- **THEN** the cache-read cost SHALL be `100 × 3.0 × 1.0 / 1_000_000 = $0.0003`

#### Scenario: cache_creation unaffected by flag
- **WHEN** `normalize_cache_read` is `true` and a request has 200 cache-creation tokens with `cache_write_1m_usd = 3.75` and `rate_cache_write = 1.0`
- **THEN** the cache-creation cost SHALL be `200 × 3.75 × 1.0 / 1_000_000 = $0.00075` (unchanged)

#### Scenario: NULL cache_read_tokens with flag enabled
- **WHEN** `normalize_cache_read` is `true` and `cache_read_tokens` is NULL
- **THEN** the cache-read cost SHALL be `0` (COALESCE to 0)

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
- **THEN** the system SHALL open a modal with 4 rate input fields (nullable, minimum 0, placeholder "1.0") and a normalize_cache_read toggle

#### Scenario: Save rates from modal
- **WHEN** an admin edits rates in the modal and clicks save
- **THEN** the system SHALL call the update assignment API with the new rate values and `normalize_cache_read` value, and update the badge display
