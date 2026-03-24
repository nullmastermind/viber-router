## Context

Viber Router is a relay proxy that routes API requests to upstream servers based on group configuration. Currently, each server has exactly one `api_key` (required), and all groups sharing a server use the same upstream credential. Collaborators need per-user key isolation: multiple users share the same group (same fallback chain) but each uses their own upstream API key for a given server.

Current data model: `servers` (id, name, base_url, api_key) → `group_servers` (group_id, server_id, priority, model_mappings) → `groups` (id, name, api_key, ...).

Current proxy flow: client sends `x-api-key` header → lookup group → iterate servers by priority → forward with server's `api_key`.

## Goals / Non-Goals

**Goals:**
- Allow clients to specify per-server upstream API keys dynamically via the `x-api-key` header
- Make server `api_key` optional (servers can exist as routing targets without a default key)
- Add `short_id` (auto-increment integer) to servers for compact identification in headers
- Skip servers that have no key (neither dynamic nor default) during failover
- Maintain full backward compatibility with existing key format

**Non-Goals:**
- Per-user key management in the database (keys are passed dynamically by the client)
- Key rotation or expiration for dynamic keys
- Rate limiting per dynamic key
- Encrypting dynamic keys in transit (beyond HTTPS)

## Decisions

### 1. Dynamic key encoding in x-api-key header

**Decision**: Encode dynamic server keys directly in the `x-api-key` header using `-rsv-` as a delimiter marker.

Format: `{group_key}-rsv-{short_id}-{server_key}[-rsv-{short_id}-{server_key}...]`

Example: `sk-vibervn-abc123-rsv-1-sk-openai-xyz-rsv-3-sk-ant-abc`

Parse algorithm:
1. Split string by `-rsv-`
2. First segment = group API key
3. Each subsequent segment: find first `-` → left side is `short_id` (integer), right side is server key

**Alternative considered**: Separate header (`x-server-keys`). Rejected because it requires client-side awareness of a custom header and breaks the single-header simplicity of the current API.

**Alternative considered**: Database-stored per-group-server key overrides. Rejected because it adds schema complexity and doesn't solve the dynamic per-user use case without also adding user management.

### 2. Server short_id as auto-increment integer

**Decision**: Add `short_id SERIAL` column to `servers` table with a UNIQUE constraint. Used in the header format instead of UUID for compactness.

**Rationale**: UUID is 36 characters; short_id is 1-4 digits. Header length matters when encoding multiple server keys.

### 3. Key resolution priority

**Decision**: For each server in the fallback chain, resolve the API key in this order:
1. Dynamic key from header (highest priority)
2. Server's default `api_key` from database
3. No key available → skip this server entirely

**Rationale**: Dynamic keys override defaults, allowing per-user isolation. Skipping keyless servers prevents sending unauthenticated requests upstream.

### 4. servers.api_key becomes nullable

**Decision**: ALTER `servers.api_key` to allow NULL. Servers can be created without a default key — they serve as routing targets that require dynamic keys.

**BREAKING**: Existing code that assumes `api_key` is always present must handle `Option<String>`. The `CreateServer` input makes `api_key` optional.

### 5. Parse failure fallback

**Decision**: If the header contains `-rsv-` but a segment fails to parse (e.g., non-numeric short_id), treat the entire string as a plain group key (backward compatible behavior). Do not partially parse.

**Rationale**: Fail safe — a malformed header should not cause unexpected routing behavior.

## Risks / Trade-offs

- **[Risk] Dynamic keys visible in logs** → Mitigation: This is an internal system; operators control logging. Document that `x-api-key` header should not be logged in production.
- **[Risk] `-rsv-` collision in upstream keys** → Mitigation: Extremely unlikely (no known provider uses this pattern). Documented as a limitation.
- **[Risk] Header length limits** → Mitigation: Most HTTP servers support 8KB+ headers. Even with 10 server keys, the header stays well under 1KB.
- **[Trade-off] No validation of dynamic keys** → The proxy does not verify dynamic keys are valid before forwarding. Invalid keys will result in upstream 401 errors, which may or may not trigger failover depending on `failover_status_codes` config. This is acceptable — the proxy is a router, not an auth system.

## Migration Plan

1. Run migration: add `short_id SERIAL UNIQUE` to `servers`, alter `api_key` to nullable
2. Existing servers get auto-assigned `short_id` values (1, 2, 3, ...)
3. Existing servers keep their `api_key` values (no data loss)
4. Deploy backend with new parser — old format keys continue to work unchanged
5. Deploy frontend with `short_id` display
6. No rollback concerns — the migration is additive (new column, relaxed constraint)
