## Purpose
TBD

## Requirements
### Requirement: Resolve API key per server in fallback chain
The system SHALL resolve the API key for each server in the fallback chain using the following priority: (1) dynamic key from parsed header matching the server's short_id, (2) server's default api_key from database, (3) no key available.

#### Scenario: Dynamic key available for server
- **WHEN** the parsed header contains a dynamic key for short_id 1 and the server in the chain has short_id 1
- **THEN** the system SHALL use the dynamic key for that server's upstream request

#### Scenario: No dynamic key, server has default key
- **WHEN** the parsed header has no dynamic key for short_id 2 and the server with short_id 2 has a non-null api_key in the database
- **THEN** the system SHALL use the server's default api_key for that server's upstream request

#### Scenario: No dynamic key and no default key — skip server
- **WHEN** the parsed header has no dynamic key for short_id 3 and the server with short_id 3 has a null api_key
- **THEN** the system SHALL skip this server and proceed to the next server in the fallback chain

#### Scenario: All servers skipped
- **WHEN** every server in the fallback chain is skipped (no dynamic key and no default key for any server)
- **THEN** the system SHALL return HTTP 401 with body `{"type":"error","error":{"type":"authentication_error","message":"No server keys configured"}}`

#### Scenario: Dynamic key for non-existent server in chain
- **WHEN** the parsed header contains a dynamic key for short_id 99 but no server in the group's chain has short_id 99
- **THEN** the system SHALL silently ignore that dynamic key entry
