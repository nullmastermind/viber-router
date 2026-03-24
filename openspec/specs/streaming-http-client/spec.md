## ADDED Requirements

### Requirement: HTTP client configured for long-running SSE streams
The shared `reqwest::Client` used for upstream proxying SHALL be constructed with explicit timeout, keepalive, and TCP settings instead of using default configuration.

#### Scenario: Long-running stream completes without timeout
- **WHEN** an upstream SSE stream runs for longer than 30 seconds
- **THEN** the proxy SHALL continue forwarding chunks until the upstream closes the stream (up to 8 hours)

#### Scenario: Connection survives idle gaps
- **WHEN** an SSE stream has a gap of 30+ seconds between chunks (e.g., during LLM thinking)
- **THEN** the proxy SHALL maintain the connection via TCP keepalive probes

#### Scenario: SSE chunks delivered without Nagle buffering
- **WHEN** the upstream sends a small SSE chunk
- **THEN** the proxy SHALL forward it immediately without waiting to batch with subsequent data (TCP_NODELAY enabled)

#### Scenario: HTTP/2 connections maintained during idle periods
- **WHEN** the upstream negotiates HTTP/2
- **THEN** the client SHALL send HTTP/2 PING frames every 30 seconds to prevent server-side idle timeout

#### Scenario: Fast connection failure for failover
- **WHEN** an upstream server is unreachable
- **THEN** the client SHALL fail the connection attempt within 10 seconds so failover can proceed to the next server
