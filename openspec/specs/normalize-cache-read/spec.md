## Purpose
TBD

## Requirements
### Requirement: normalize_cache_read flag on group_servers
The `group_servers` table SHALL have a `normalize_cache_read BOOLEAN NOT NULL DEFAULT false` column. When `true`, cache-read tokens for that server assignment SHALL be priced at the input rate instead of the cache-read rate.

#### Scenario: Default value
- **WHEN** a new server is assigned to a group without specifying `normalize_cache_read`
- **THEN** the assignment SHALL have `normalize_cache_read = false`

#### Scenario: Flag persisted via update API
- **WHEN** an admin sends `PUT /api/admin/groups/:id/servers/:sid` with `{ "normalize_cache_read": true }`
- **THEN** the system SHALL update the assignment and return the updated record with `normalize_cache_read: true`

#### Scenario: Flag returned in group detail
- **WHEN** an admin fetches `GET /api/admin/groups/:id`
- **THEN** each server entry in the `servers` array SHALL include `normalize_cache_read: true` or `normalize_cache_read: false`

### Requirement: normalize_cache_read toggle in Cost Rates modal
The Cost Rates modal in the group detail admin UI SHALL include a toggle for `normalize_cache_read`.

#### Scenario: Toggle reflects current value
- **WHEN** an admin opens the Cost Rates modal for a server assignment
- **THEN** the toggle SHALL reflect the current `normalize_cache_read` value for that assignment

#### Scenario: Toggle saved with rates
- **WHEN** an admin changes the `normalize_cache_read` toggle and clicks Save
- **THEN** the system SHALL call the update assignment API with the new `normalize_cache_read` value

### Requirement: normalize_cache_read flows through proxy cache
The `GroupServerDetail` struct (used in the Redis-cached `GroupConfig`) SHALL include `normalize_cache_read`. The proxy SHALL read this field when processing requests.

#### Scenario: Flag available at request time
- **WHEN** a request is routed to a server with `normalize_cache_read = true`
- **THEN** the proxy SHALL pass `normalize_cache_read = true` to `calculate_cost()` without a DB lookup

#### Scenario: Cache deserialization with missing field
- **WHEN** a cached `GroupConfig` was serialized before the field was added (field absent in JSON)
- **THEN** `normalize_cache_read` SHALL deserialize to `false` (via `#[serde(default)]`)
