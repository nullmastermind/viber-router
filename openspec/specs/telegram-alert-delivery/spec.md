## Purpose
TBD

## Requirements
### Requirement: Telegram alert on circuit breaker trip
The system SHALL send a Telegram alert when a circuit breaker trips (server auto-disabled). The alert SHALL use the same delivery mechanism as upstream error alerts (settings, cooldown, chat IDs). The cooldown key SHALL be `tg:cooldown:cb:{group_id}:{server_id}` with TTL = `alert_cooldown_mins * 60`.

#### Scenario: Circuit breaker trips — alert sent
- **WHEN** a server's error count reaches `cb_max_failures` and no cooldown key exists for this circuit event
- **THEN** the system SHALL send a Telegram alert with message containing: server name, group name, error count, window seconds, and cooldown seconds

#### Scenario: Circuit breaker trips — cooldown active
- **WHEN** a circuit breaker trips but `tg:cooldown:cb:{group_id}:{server_id}` already exists
- **THEN** the system SHALL NOT send a duplicate alert

### Requirement: Telegram alert on circuit breaker re-enable
The system SHALL send a Telegram alert when a circuit-broken server is re-enabled. Detection uses check-on-next-request: when the proxy checks `cb:open:{group_id}:{server_id}` and the key does not exist, but a one-time marker `cb:realerted:{group_id}:{server_id}` does not exist, the system SHALL send the re-enable alert and set the marker.

#### Scenario: Server re-enabled — alert sent on next request
- **WHEN** a request arrives, `cb:open:{group_id}:{server_id}` has expired, and `cb:realerted:{group_id}:{server_id}` does not exist
- **THEN** the system SHALL send a Telegram alert indicating the server is back online and SET `cb:realerted:{group_id}:{server_id}` with TTL=60 to prevent duplicate alerts

#### Scenario: Server re-enabled — already alerted
- **WHEN** a request arrives, `cb:open` has expired, but `cb:realerted:{group_id}:{server_id}` exists
- **THEN** the system SHALL NOT send another re-enable alert
