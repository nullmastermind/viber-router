## Context

Viber Router proxies Anthropic API requests through a failover waterfall of upstream servers. Each group has an ordered list of servers; the proxy tries them in priority order until one succeeds. The `/v1/messages/count_tokens` endpoint currently follows this same waterfall with no special handling.

Some groups need token-counting routed to a dedicated server (lower latency, separate quota) before falling back to the normal chain. Today there is no per-route server override.

The `GroupConfig` struct is cached in Redis (keyed by group API key) and contains all data the proxy needs at request time. The `invalidate_groups_by_server` function invalidates cache entries for groups referencing a given server via the `group_servers` join table.

## Goals / Non-Goals

**Goals:**
- Allow per-group configuration of a default server for `/v1/messages/count_tokens`
- Try the default server first; on failure, fall back to the normal failover chain (skipping the default server if it appears in the chain)
- Support dynamic key resolution via `short_id` for the default server
- Support separate model mappings for count-tokens requests
- Embed the resolved server detail in `GroupConfig` for single-lookup caching
- Expose configuration in the admin UI (Group detail page)

**Non-Goals:**
- Generic per-route server overrides (only count_tokens for now)
- Separate cache key/TTL for count-tokens server info
- TTFT measurement for count_tokens (non-streaming endpoint)

## Decisions

### 1. Embed count-tokens server in GroupConfig (not separate cache)

**Choice**: Add `count_tokens_server: Option<CountTokensServer>` to `GroupConfig`, resolved during `resolve_group_config`.

**Alternatives considered**:
- Separate Redis key (`ct_server:{id}`) — rejected because it adds a second Redis lookup per count_tokens request and a second invalidation path to maintain.

**Rationale**: The existing `GroupConfig` cache pattern is well-established. Embedding keeps one lookup, one invalidation path, and zero new cache functions.

### 2. New struct `CountTokensServer` for the embedded detail

Fields: `server_id: Uuid`, `short_id: i32`, `server_name: String`, `base_url: String`, `api_key: Option<String>`, `model_mappings: serde_json::Value`.

The `model_mappings` here comes from `groups.count_tokens_model_mappings`, not from `group_servers`. The rest comes from the `servers` table.

### 3. Path check is hardcoded

The proxy handler checks `original_uri.path() == "/v1/messages/count_tokens"` before the failover loop. No configuration table for route overrides.

### 4. Key resolution follows existing pattern

For the default server: check `parsed.dynamic_keys.get(&server.short_id)` first, then fall back to `server.api_key`. If neither exists, skip the default server and proceed to the normal chain.

### 5. Failover behavior matches group config

The default server attempt uses the same `failover_status_codes` as the group. Non-failover errors (e.g., 400) are returned directly to the client.

### 6. Skip logic in failover chain

After the default server attempt fails, the failover loop skips any server with `server_id == count_tokens_server.server_id` to avoid a redundant retry.

### 7. DB schema: nullable FK with ON DELETE SET NULL

`count_tokens_server_id UUID REFERENCES servers(id) ON DELETE SET NULL` — if the referenced server is deleted, the field auto-clears and the group falls back to normal routing.

### 8. UpdateGroup pattern follows ttft_timeout_ms precedent

The `CASE WHEN $N THEN $M ELSE col END` SQL pattern (used for `ttft_timeout_ms`) is reused for both `count_tokens_server_id` and `count_tokens_model_mappings` to support independent nullable updates.

### 9. Cache invalidation extended

`invalidate_groups_by_server` query gains `OR g.count_tokens_server_id = $1` to cover groups using a server as count-tokens default.

## Risks / Trade-offs

- **Slightly larger GroupConfig cache blob** → Negligible; one extra optional struct field.
- **`resolve_group_config` gains a conditional JOIN** → Only when `count_tokens_server_id IS NOT NULL`; no impact on groups without the feature configured.
- **`delete_server` silently clears count_tokens default** → `ON DELETE SET NULL` handles DB integrity. The admin UI does not warn that deleting a server removes it as a count-tokens default. Accepted trade-off for simplicity.
- **Backward compatibility of cached GroupConfig** → Existing cached entries without the new field will fail deserialization and trigger a DB re-fetch. This is the established self-healing pattern (same as ttft_timeout_ms rollout).
