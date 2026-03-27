## Why

Each server in a group has different capacity. Currently there is no way to limit how many requests a server receives within a time window. When a high-traffic group sends too many requests to a lower-capacity server, that server becomes overloaded. Admins need per group-server rate limiting so the proxy automatically skips servers that have reached their request quota and routes to the next server in the chain.

## What Changes

- Add per group-server rate limiting with two configurable fields: `max_requests` (max number of requests) and `rate_window_seconds` (time window in seconds)
- Proxy checks a Redis counter before sending each request; if the server has reached its limit within the current window, it skips to the next server in the chain
- Counter uses optimistic increment (INCR before sending the request)
- When all servers in the chain are rate-limited, return 429 "Rate limit exceeded"
- Redis failure during rate limit check fails open (request proceeds)
- Admin UI shows rate limit configuration in the Edit Server dialog and a badge on the server list

## Capabilities

### New Capabilities
- `server-rate-limit`: Per group-server request rate limiting with Redis-based sliding window counter, configurable max requests and window duration

### Modified Capabilities
- `group-server-assignment`: Add `max_requests` and `rate_window_seconds` nullable fields to the group-server assignment, with all-or-nothing validation (both set or both NULL, values >= 1)
- `proxy-engine`: Check rate limit in the failover waterfall before sending request; skip server if limit reached; return 429 "Rate limit exceeded" when all servers exhausted due to rate limiting
- `admin-ui`: Add rate limit configuration section in Edit Server dialog and rate limit badge on server list

## Impact

- **Database**: New migration adding `max_requests` and `rate_window_seconds` columns to `group_servers` table
- **Backend**: New `rate_limiter.rs` module (mirrors `circuit_breaker.rs`), changes to proxy handler loop, changes to admin group_servers API handlers and models
- **Frontend**: Changes to `GroupDetailPage.vue` (Edit Server dialog + badge), `groups.ts` store (types)
- **Redis**: New key pattern `rl:{group_id}:{server_id}` for rate limit counters
- **Cache**: No changes needed — fields flow through existing `GroupServerDetail` → `GroupConfig` path
