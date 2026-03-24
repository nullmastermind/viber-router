## MODIFIED Requirements

### Requirement: Group identification via x-api-key header
The system SHALL parse the `x-api-key` header to extract the group API key (and optional dynamic server keys), then identify the target group by looking up the extracted group key against group API keys.

#### Scenario: Valid API key (plain format)
- **WHEN** a request has `x-api-key` with value `sk-vibervn-abc123` matching an active group's API key
- **THEN** the system SHALL use that group's configuration for routing with no dynamic keys

#### Scenario: Valid API key (with dynamic keys)
- **WHEN** a request has `x-api-key` with value `sk-vibervn-abc123-rsv-1-sk-openai-xyz` where `sk-vibervn-abc123` matches an active group's API key
- **THEN** the system SHALL use that group's configuration for routing and pass the dynamic key map `{1: "sk-openai-xyz"}` to the key resolution step

#### Scenario: Unknown API key
- **WHEN** a request has `x-api-key` whose extracted group key does not match any group
- **THEN** the system SHALL return HTTP 401 with Anthropic-compatible error JSON: `{"type":"error","error":{"type":"authentication_error","message":"Invalid API key"}}`

#### Scenario: Missing x-api-key header
- **WHEN** a request to `/v1/*` has no `x-api-key` header
- **THEN** the system SHALL return HTTP 401 with Anthropic-compatible error JSON

#### Scenario: Inactive group
- **WHEN** a request has `x-api-key` whose extracted group key matches a group with `is_active: false`
- **THEN** the system SHALL return HTTP 403 with Anthropic-compatible error JSON: `{"type":"error","error":{"type":"permission_error","message":"API key is disabled"}}`

### Requirement: Upstream API key substitution
The system SHALL replace the client's `x-api-key` header with the resolved API key for the target upstream server before forwarding. The resolved key comes from the key resolution step (dynamic key > server default key).

#### Scenario: Key swap with dynamic key
- **WHEN** forwarding a request to server S1 (short_id=1) and the client provided dynamic key `sk-openai-xyz` for short_id 1
- **THEN** the `x-api-key` header in the forwarded request SHALL be `sk-openai-xyz`

#### Scenario: Key swap with server default key
- **WHEN** forwarding a request to server S1 (short_id=1) and no dynamic key was provided for short_id 1, but S1 has default api_key `sk-ant-upstream-xxx`
- **THEN** the `x-api-key` header in the forwarded request SHALL be `sk-ant-upstream-xxx`

### Requirement: Priority-based failover waterfall
The system SHALL try upstream servers in priority order (lowest priority number first), skipping servers that have no resolved API key. If a server returns a status code that is in the group's `failover_status_codes` set, the system SHALL try the next server with a resolved key. This continues until a server returns a non-failover status code or all servers are exhausted.

#### Scenario: First server succeeds
- **WHEN** server at priority 1 has a resolved key and returns HTTP 200
- **THEN** the system SHALL return that response to the client without trying other servers

#### Scenario: First server has no key, second server succeeds
- **WHEN** server at priority 1 has no resolved key (skipped) and server at priority 2 has a resolved key and returns HTTP 200
- **THEN** the system SHALL return the response from server at priority 2

#### Scenario: First server fails with failover code
- **WHEN** server at priority 1 has a resolved key and returns HTTP 429 (in failover_status_codes)
- **THEN** the system SHALL try the next server that has a resolved key

#### Scenario: All servers with keys exhausted
- **WHEN** all servers with resolved keys return failover status codes or connection errors
- **THEN** the system SHALL return HTTP 429 with `Retry-After: 30` header and body `{"type":"error","error":{"type":"overloaded_error","message":"All upstream servers unavailable"}}`

#### Scenario: All servers skipped (no keys)
- **WHEN** no server in the chain has a resolved key (all skipped)
- **THEN** the system SHALL return HTTP 401 with body `{"type":"error","error":{"type":"authentication_error","message":"No server keys configured"}}`
