## Why

The project needs a relay/proxy layer for Anthropic's `/v1/*` API endpoints that groups multiple upstream servers with priority-based failover. When one upstream server returns a configured error status code, the relay automatically tries the next server in priority order. This enables high availability and seamless switching between multiple Anthropic-compatible API providers with different model naming conventions.

## What Changes

- Add proxy engine in Axum that intercepts all `/v1/*` requests, authenticates via `x-api-key` header to identify the group, and forwards to upstream servers with failover
- Add `servers` table for shared upstream server definitions (name, base_url, api_key)
- Add `groups` table with auto-generated API keys (`sk-vibervn-*`), configurable failover status codes, and active/inactive status
- Add `group_servers` junction table with priority ordering and per-group-server model name mappings (JSONB)
- Add embedded SQLx migrations that auto-run on startup
- Add Redis caching layer for group config lookup by API key, with write-through invalidation
- Add admin REST API (CRUD for servers, groups, group-server assignments) protected by `ADMIN_TOKEN` env var
- Add admin UI (Vue 3 + Quasar) with server-side paginated tables, search/filter, bulk operations (activate/deactivate, delete, bulk assign server), and drag-to-reorder server priority
- Support both streaming (SSE) and non-streaming proxy modes
- When all servers in a group fail, return HTTP 429 with `Retry-After: 30` header and Anthropic-compatible error JSON

## Capabilities

### New Capabilities
- `proxy-engine`: Core relay logic — request interception, API key-based group lookup, upstream forwarding with model name transformation, priority-based failover waterfall, SSE streaming passthrough
- `server-management`: CRUD operations for shared upstream server definitions (name, base_url, api_key), used by admin API and UI
- `group-management`: CRUD operations for groups (name, auto-generated sk-vibervn-* API key, failover status codes, active/inactive toggle), including bulk operations (activate/deactivate, delete, bulk assign server)
- `group-server-assignment`: Managing the relationship between groups and servers — priority ordering, per-group-server model name mappings, drag-to-reorder
- `config-cache`: Redis caching of group configuration keyed by API key, with write-through invalidation on admin changes
- `admin-auth`: Simple token-based authentication using ADMIN_TOKEN env var, protecting both UI and programmatic API access
- `admin-ui`: Vue 3 + Quasar admin interface with server-side paginated tables, search by name, filter by active/inactive and by server, bulk operations, and professional UX for high-volume group management (1000+ groups/day)

### Modified Capabilities
- `server-setup`: Add ADMIN_TOKEN to required env vars, add reqwest HTTP client dependency, mount proxy and admin routes

## Impact

- Backend: New proxy handler, admin CRUD handlers, migration files, Redis cache module, reqwest dependency for upstream HTTP calls
- Frontend: Replace boilerplate IndexPage with full admin UI (servers page, groups page with detail view)
- Database: 3 new tables (servers, groups, group_servers) with indexes on api_key, name, is_active
- Redis: Used for config caching (group lookup by API key)
- Config: New ADMIN_TOKEN env var required
- API surface: All `/v1/*` routes proxied; `/api/admin/*` routes for management
