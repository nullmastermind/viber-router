## Context

The viber-router proxy forwards API requests (including SSE streaming) to upstream LLM providers. The current HTTP client (`reqwest::Client::new()`) uses default settings that are unsuitable for long-running SSE streams — 30s total timeout, no TCP keepalive, Nagle buffering enabled. This causes streams to drop mid-response when proxied, while direct connections work fine.

## Goals / Non-Goals

**Goals:**
- Eliminate streaming drops caused by HTTP client misconfiguration
- Support long-running agent streams (up to 8 hours)
- Keep connections alive through NAT/firewalls during idle gaps in SSE streams

**Non-Goals:**
- Per-server HTTP client configuration (single shared client is sufficient)
- HTTP version pinning (let reqwest auto-negotiate HTTP/1.1 or HTTP/2)
- Retry logic for mid-stream failures (separate concern)

## Decisions

### 1. Single configured client replacing `reqwest::Client::new()`

**Decision**: Replace the default client in `main.rs` with a `Client::builder()` using explicit timeout and keepalive settings.

**Rationale**: The change is minimal (one location), affects all upstream connections uniformly, and requires no architectural changes.

**Alternative considered**: Separate clients per upstream server or per protocol version. Rejected — adds complexity with no clear benefit since all upstreams need the same streaming-friendly settings.

### 2. Client settings

| Setting | Value | Why |
|---------|-------|-----|
| `timeout` | 8 hours | Long-running agent streams can last hours |
| `connect_timeout` | 10 seconds | Fast-fail on unreachable servers (failover kicks in) |
| `pool_idle_timeout` | 1 hour | Reuse connections across requests |
| `tcp_keepalive` | 30 seconds | Prevent NAT/firewall from dropping idle connections |
| `tcp_nodelay` | true | Disable Nagle — send SSE chunks immediately |
| `http2_keep_alive_interval` | 30 seconds | HTTP/2 PING frames during streaming gaps |
| `http2_keep_alive_timeout` | 10 seconds | Fail fast if PING not acknowledged |
| `http2_keep_alive_while_idle` | true | Keep pinging even when no active streams |

## Risks / Trade-offs

- [8-hour timeout] A stuck connection could hang for 8 hours before timing out → Acceptable because upstream APIs have their own timeouts and clients will disconnect, closing the proxy connection.
- [HTTP/2 keep-alive on HTTP/1.1 servers] The `http2_keep_alive_*` settings are no-ops when the connection negotiates HTTP/1.1 → No risk, just unused config.
