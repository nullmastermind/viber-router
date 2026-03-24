## Why

SSE streaming responses through the viber-router proxy are intermittently cut off mid-stream. Direct connections to upstream APIs do not exhibit this issue. The root cause is that `reqwest::Client::new()` uses default settings: a 30-second total timeout (kills long streams), no TCP keepalive (connections dropped by NAT/firewalls during idle gaps), and no `tcp_nodelay` (Nagle's algorithm buffers small SSE chunks).

## What Changes

- Replace `reqwest::Client::new()` with a configured `reqwest::Client::builder()` that sets appropriate timeouts, TCP keepalive, TCP nodelay, and HTTP/2 keep-alive pings for reliable long-running SSE streaming through the proxy.

## Capabilities

### New Capabilities

- `streaming-http-client`: Configure the shared reqwest HTTP client with timeout, keepalive, and TCP settings optimized for proxying long-running SSE streams.

### Modified Capabilities

(none)

## Impact

- `viber-router-api/src/main.rs` — HTTP client construction (line 49)
- No API changes, no new dependencies, no breaking changes
