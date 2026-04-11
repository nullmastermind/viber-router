## Why

Admins have no visibility into which clients are accessing a group, and no way to block specific clients without revoking their API key entirely. Adding user-agent tracking and blocking gives admins fine-grained access control at the client level without disrupting other users sharing the same key.

## What Changes

- New DB migration (038) adds `group_user_agents` and `group_blocked_user_agents` tables
- Proxy records unique user-agents per group into Redis + DB on each request (fire-and-forget)
- Proxy checks blocked user-agents before forwarding — returns 403 if matched
- `GroupConfig` gains a `blocked_user_agents` field loaded during config resolution
- New Redis helper `add_group_ua` for atomic SADD-based deduplication
- New admin API routes under `/{group_id}/user-agents` for listing and managing blocked UAs
- New "User Agents" tab in GroupDetailPage with autocomplete-based blocking UI

## Capabilities

### New Capabilities

- `user-agent-tracking`: Records unique user-agents seen per group via Redis deduplication and DB persistence
- `user-agent-blocking`: Allows admins to block specific user-agents per group; proxy enforces blocks with 403 responses
- `user-agent-admin-ui`: Admin UI tab on group detail page for viewing recorded UAs and managing the blocked list

### Modified Capabilities

- `proxy-engine`: Proxy handler gains UA recording (fire-and-forget) and UA block check after is_active validation
- `group-management`: GroupConfig gains `blocked_user_agents` field; resolve_group_config loads blocked list from DB

## Impact

- Backend: `proxy.rs`, `cache.rs`, `models/group_server.rs`, new `routes/admin/group_user_agents.rs`, `routes/admin/mod.rs`, `routes/admin/groups.rs`
- Database: new migration 038 with two tables and an index
- Frontend: `stores/groups.ts` (4 new store functions), `pages/GroupDetailPage.vue` (new tab)
- No breaking changes to existing API contracts
- Cache invalidation: blocked UA mutations call `invalidate_group_all_keys` so changes take effect immediately
