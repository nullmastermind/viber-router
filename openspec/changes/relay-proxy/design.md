## Context

Viber Router is a monorepo with a Vue 3 / Quasar frontend and Rust (Axum) backend. The backend currently has only a health endpoint with PostgreSQL and Redis connections established. The frontend is boilerplate Quasar scaffolding.

The system needs to become an Anthropic API relay proxy — accepting standard Anthropic API requests and forwarding them to configurable upstream servers with automatic failover. The admin UI manages server and group configuration. Groups are created at high volume (1000+/day), requiring professional data management UX.

## Goals / Non-Goals

**Goals:**
- Drop-in Anthropic API replacement: clients use standard `x-api-key` header and `/v1/*` paths
- Priority-based failover waterfall across all servers in a group
- Per-group-server model name transformation in request body
- Both SSE streaming and non-streaming proxy support
- Redis-cached config for fast per-request group lookup
- Admin UI with server-side pagination, search, filter, bulk operations for high-volume management
- Embedded auto-migrations on startup

**Non-Goals:**
- Rate limiting (not in scope)
- Request/response logging or analytics
- Usage tracking per group
- Load balancing (round-robin, least-connections) — this is strictly priority-based failover
- Upstream health checking (proactive) — only reactive failover on error responses
- Timeout management — upstream servers handle their own timeouts
- Response body modification — passthrough only

## Decisions

### 1. Routing: API key-based group identification

**Decision**: Use `x-api-key` header to identify the group. No group ID in URL path.

**Rationale**: Makes Viber Router a true drop-in replacement for the Anthropic API. Clients only need to change the host and API key — no URL structure changes. Any Anthropic SDK works without modification.

**Alternative considered**: Group ID in URL path (`/group-xxx/v1/messages`). Rejected because it breaks drop-in compatibility and requires client-side URL changes.

### 2. Data model: Shared servers with junction table

**Decision**: Three tables — `servers` (shared definitions), `groups` (with API key), `group_servers` (junction with priority + model_mappings JSONB).

```
servers (id, name, base_url, api_key, created_at, updated_at)
   │
   └──< group_servers (group_id, server_id, priority, model_mappings, created_at)
   │
groups (id, name, api_key, failover_status_codes, is_active, created_at, updated_at)
```

**Rationale**: Servers are shared resources (e.g., a global fallback server referenced by many groups). The junction table allows per-group-server model mappings and priority ordering. JSONB for model_mappings keeps the schema simple — no need for a separate mapping table.

**Alternative considered**: Inline server config per group (denormalized). Rejected because updating a shared server (e.g., rotating its API key) would require updating every group that uses it.

### 3. Proxy implementation: reqwest with body buffering

**Decision**: Use `reqwest` as the HTTP client. Buffer the full request body in memory before forwarding.

**Rationale**: Body must be buffered because (a) model name transformation requires parsing and modifying the JSON body, and (b) failover requires resending the same body to the next server. Anthropic API request bodies are text prompts — typically small (< 1MB). No body size limit needed.

For streaming responses: proxy the upstream SSE stream directly to the client using Axum's streaming response. Failover only happens before the stream starts (on connection error or error status code).

**Alternative considered**: Streaming the request body through without buffering. Not possible due to failover retry requirement.

### 4. Failover logic: Waterfall with configurable status codes

**Decision**: On each request, try servers in priority order. If the response status code is in the group's `failover_status_codes` set, try the next server. Continue until success or all servers exhausted.

When all servers fail: return HTTP 429 with `Retry-After: 30` header and Anthropic-compatible error body:
```json
{"type":"error","error":{"type":"overloaded_error","message":"All upstream servers unavailable"}}
```

**Rationale**: 429 signals the client to retry (standard behavior for Anthropic SDKs). The Retry-After header gives clients a backoff hint. Anthropic-compatible error format ensures SDK error handling works correctly.

### 5. Config caching: Redis with write-through invalidation

**Decision**: Cache the full group config (including server list, priorities, model mappings) in Redis, keyed by API key. On any admin write (create/update/delete group, change server assignments), invalidate the relevant cache entries immediately.

**Cache structure**: Key = `group:config:{api_key}`, Value = serialized JSON of group + servers + mappings, no TTL (invalidated explicitly).

**Rationale**: Every proxy request needs a group config lookup. With 30K+ groups, hitting PostgreSQL on every request is a bottleneck. Write-through ensures consistency — admin changes are reflected immediately.

**Alternative considered**: TTL-based expiry. Rejected because admin expects changes to take effect immediately, and stale config could route to wrong servers.

### 6. Admin auth: ADMIN_TOKEN env var

**Decision**: Single `ADMIN_TOKEN` environment variable. All admin API endpoints (both UI-facing and programmatic) require `Authorization: Bearer <token>` header matching this value. UI stores the token in localStorage after login.

**Rationale**: Simple, sufficient for internal tooling. No user management overhead. Single token shared across UI and programmatic access.

### 7. API key generation: sk-vibervn- prefix with random suffix

**Decision**: Auto-generate on group creation. Format: `sk-vibervn-` + 24 random alphanumeric characters. Support regeneration (generates new key, invalidates old cache entry).

**Rationale**: Prefix makes keys identifiable as Viber Router keys. 24 random chars provides sufficient entropy. Follows Anthropic's `sk-ant-` convention.

### 8. Migration: Embedded in binary

**Decision**: Use `sqlx::migrate!()` macro to embed migrations at compile time. Run automatically on application startup before binding the HTTP listener.

**Rationale**: Single binary deployment — no separate migration step needed. Migrations run before the server accepts traffic, ensuring schema is always up to date.

### 9. Frontend admin UI: Server-side pagination with Quasar QTable

**Decision**: Use Quasar's QTable component with server-side pagination, sorting, and filtering. Backend provides paginated API endpoints with query parameters for page, limit, search, filters.

**Rationale**: With 30K+ groups, client-side pagination is not viable. QTable has built-in support for server-side mode. Bulk operations use checkbox selection with batch API calls.

## Risks / Trade-offs

- **[Body buffering memory]** Large request bodies are buffered entirely in memory for failover retry. → Mitigation: Anthropic API requests are text-based and typically small. No practical risk for the intended use case.
- **[Single ADMIN_TOKEN]** If token is compromised, all admin access is exposed. → Mitigation: Acceptable for internal tooling. Token can be rotated by changing env var and restarting.
- **[Redis single point of failure for cache]** If Redis is down, every request hits PostgreSQL. → Mitigation: Fallback to DB query on Redis miss/error. Redis is not required for correctness, only performance.
- **[No upstream health checks]** Bad servers are only detected when requests fail. → Mitigation: Failover waterfall handles this reactively. Proactive health checks can be added later if needed.
- **[Embedded migrations risk]** If a migration fails on startup, the server won't start. → Mitigation: Migrations should be tested before deployment. Rollback requires a code change and redeploy.
