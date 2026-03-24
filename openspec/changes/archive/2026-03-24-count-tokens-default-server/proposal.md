## Why

The `/v1/messages/count_tokens` endpoint currently follows the same failover waterfall as all other proxy routes. Some groups need to route token-counting requests to a specific server (e.g., one with lower latency or a dedicated quota) before falling back to the normal chain. There is no way to configure this today.

## What Changes

- Add `count_tokens_server_id` (nullable UUID FK to `servers`) and `count_tokens_model_mappings` (JSONB) columns to the `groups` table.
- When a request hits `/v1/messages/count_tokens` and the group has a `count_tokens_server_id` configured, try that server first. If it fails (failover status code or connection error), continue with the normal failover chain but skip the default server to avoid a redundant retry.
- Dynamic key resolution via `short_id` is supported for the default server.
- Embed the resolved count-tokens server detail inside `GroupConfig` so it is cached alongside existing proxy config — no separate Redis key needed.
- Extend `invalidate_groups_by_server` to also invalidate groups referencing a server as `count_tokens_server_id`.
- Add a "Count Tokens Server" dropdown and model mappings editor in the Group detail admin page.

## Capabilities

### New Capabilities
- `count-tokens-routing`: Per-group default server selection for `/v1/messages/count_tokens`, with failover back to the normal chain (skipping the default server).

### Modified Capabilities
- `proxy-engine`: The proxy handler gains a path-specific pre-chain step for count_tokens requests.
- `group-management`: The Group model, API, and admin UI gain two new fields (`count_tokens_server_id`, `count_tokens_model_mappings`).
- `config-cache`: `GroupConfig` embeds an optional count-tokens server detail; `invalidate_groups_by_server` query is extended.

## Impact

- **Database**: Migration adds two columns to `groups` with FK constraint (`ON DELETE SET NULL`).
- **Backend models**: `Group`, `GroupListItem`, `GroupConfig`, `UpdateGroup` structs gain new fields.
- **Proxy handler**: New path-check branch before the failover loop.
- **Cache**: `resolve_group_config` fetches count-tokens server detail; `invalidate_groups_by_server` query extended.
- **Admin API**: `update_group` handler accepts new fields.
- **Frontend**: `GroupDetailPage.vue` adds dropdown + model mappings UI; store types updated.
