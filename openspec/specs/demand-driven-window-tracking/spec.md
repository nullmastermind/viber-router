## ADDED Requirements

### Requirement: Window start key lifecycle
The system SHALL store the start of an active subscription window in Redis under the key `sub_window_start:{sub_id}` as a Unix epoch integer (i64). The key SHALL have a TTL of exactly `reset_hours * 3600` seconds. When the TTL expires, no window is active for that subscription.

#### Scenario: First request creates window
- **WHEN** no `sub_window_start:{sub_id}` key exists in Redis and a request is processed for a subscription with `reset_hours` set
- **THEN** the system SHALL SET `sub_window_start:{sub_id}` to the current Unix epoch with NX and EX = `reset_hours * 3600`, and return the stored epoch

#### Scenario: Subsequent requests in same window
- **WHEN** `sub_window_start:{sub_id}` already exists in Redis and a request is processed for the same subscription
- **THEN** the SETNX operation SHALL be a no-op and the system SHALL return the existing epoch value

#### Scenario: Window expires with no new request
- **WHEN** the TTL on `sub_window_start:{sub_id}` reaches zero with no new request
- **THEN** the key SHALL be automatically removed by Redis and no new window SHALL be created until the next request arrives

#### Scenario: Redis unavailable during window creation
- **WHEN** Redis is unavailable when `ensure_window_start` is called
- **THEN** the system SHALL log a warning and return `None`, skipping cost counter updates for that request

### Requirement: No window active means zero cost
The system SHALL treat a missing `sub_window_start:{sub_id}` key as "no active window" and return `0.0` for cost reads on that subscription.

#### Scenario: Cost read with no active window
- **WHEN** `sub_window_start:{sub_id}` does not exist in Redis and `get_total_cost` or `get_model_cost` is called
- **THEN** the system SHALL return `0.0` without querying any cost counter key

#### Scenario: Cost read with active window
- **WHEN** `sub_window_start:{sub_id}` exists with epoch `ws` and `get_total_cost` is called
- **THEN** the system SHALL read from `sub_cost:{sub_id}:ws:{ws}` and return the stored value (or rebuild from DB on miss)
