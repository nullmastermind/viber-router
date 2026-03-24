## Why

Collaborators have multiple users per upstream server, each with their own API key, but share the same group (same fallback chain). Currently each server has exactly one API key, so all groups using that server share the same upstream credential. This prevents per-user key isolation within a shared routing group.

## What Changes

- **BREAKING**: `servers.api_key` becomes nullable — servers can exist without a default key
- Add `servers.short_id` (auto-increment integer) for compact server identification in headers
- Parse `x-api-key` header to extract dynamic per-server keys using `-rsv-` delimiter format: `sk-vibervn-{group}-rsv-{short_id}-{server_key}[-rsv-{id}-{key}...]`
- Key resolution priority per server in fallback chain: dynamic key > server default key > skip server
- When all servers in chain are skipped (no keys available), return HTTP 401 "No server keys configured"
- Display `short_id` everywhere servers are shown in the admin UI, with copy button

## Capabilities

### New Capabilities
- `dynamic-key-parsing`: Parse x-api-key header to extract group key and per-server dynamic keys using `-rsv-{short_id}-{key}` format
- `key-resolution`: Resolve which API key to use per server in fallback chain (dynamic > default > skip)

### Modified Capabilities
- `server-management`: servers.api_key becomes nullable, add short_id auto-increment field, UI shows short_id with copy button
- `proxy-engine`: Integrate dynamic key parsing and key resolution into the proxy request flow
- `group-server-assignment`: GroupServerDetail.api_key becomes optional, UI displays server short_id

## Impact

- **Database**: Migration to alter `servers` table (nullable api_key, add short_id serial column)
- **Backend**: New parser module, modified proxy handler, updated server/group models and queries
- **Frontend**: ServersPage, GroupDetailPage, GroupsPage — show short_id with copy button everywhere servers appear
- **API compatibility**: Existing x-api-key headers without `-rsv-` continue to work unchanged (backward compatible)
- **Limitation**: Upstream server keys must not contain the literal string `-rsv-` (documented)
