## ADDED Requirements

### Requirement: Parse dynamic server keys from x-api-key header
The system SHALL parse the `x-api-key` header to extract a group API key and zero or more dynamic per-server keys. The format is `{group_key}-rsv-{short_id}-{server_key}` where multiple `-rsv-{short_id}-{server_key}` segments may be appended.

#### Scenario: Plain group key (no dynamic keys)
- **WHEN** the `x-api-key` header value is `sk-vibervn-abc123` (contains no `-rsv-` substring)
- **THEN** the system SHALL extract group_key as `sk-vibervn-abc123` and dynamic_keys as an empty map

#### Scenario: Single dynamic server key
- **WHEN** the `x-api-key` header value is `sk-vibervn-abc123-rsv-1-sk-openai-xyz`
- **THEN** the system SHALL extract group_key as `sk-vibervn-abc123`, and dynamic_keys as `{1: "sk-openai-xyz"}`

#### Scenario: Multiple dynamic server keys
- **WHEN** the `x-api-key` header value is `sk-vibervn-abc123-rsv-1-sk-openai-xyz-rsv-3-sk-ant-abc`
- **THEN** the system SHALL extract group_key as `sk-vibervn-abc123`, and dynamic_keys as `{1: "sk-openai-xyz", 3: "sk-ant-abc"}`

#### Scenario: Malformed segment (non-numeric short_id)
- **WHEN** the `x-api-key` header value contains `-rsv-` but a segment has a non-numeric short_id (e.g., `sk-vibervn-abc123-rsv-notanumber-sk-key`)
- **THEN** the system SHALL treat the entire header value as a plain group key with no dynamic keys

#### Scenario: Segment with no key after short_id
- **WHEN** a `-rsv-` segment contains only a short_id with no key (e.g., `sk-vibervn-abc123-rsv-1`)
- **THEN** the system SHALL treat the entire header value as a plain group key with no dynamic keys
