## Context

The proxy engine routes requests through a prioritized failover chain of servers within a group. Each server in `group_servers` already carries per-assignment config (circuit breaker thresholds, rate limits, token limits). The failover loop already has multiple skip conditions: no API key, circuit breaker open, rate limited, max_input_tokens exceeded.

Currently there is no way to restrict a server to specific models. If a group mixes servers that only support certain model families, every server is attempted regardless of the requested model, leading to upstream errors that consume failover budget unnecessarily.

The `GroupConfig` struct (with its `servers: Vec<GroupServerDetail>`) is serialized to Redis and cached per API key. Any new field on `GroupServerDetail` is automatically included in the cache on next miss.

## Goals / Non-Goals

**Goals:**
- Add `supported_models: Vec<String>` to the `group_servers` table and all related structs
- Skip servers in the failover loop when the requested model is not in their `supported_models` list
- Treat a key in `model_mappings` as implicitly supported (no duplication required)
- Expose the field in admin GET/PUT endpoints and the frontend edit dialog
- Remain fully backward compatible (empty list = no filtering)

**Non-Goals:**
- No new error type or HTTP status for model-unsupported skips — silent skip only
- No group-level model filtering changes (that is handled by `allowed_models` junction table)
- No validation that listed model names exist in the `models` table — free-text is acceptable

## Decisions

### 1. Storage: TEXT[] column vs junction table

Chosen: `TEXT[] NOT NULL DEFAULT '{}'` column on `group_servers`.

The `allowed_models` group-level feature uses a junction table because it drives authorization (403 responses) and needs relational integrity. `supported_models` is a routing hint on a per-assignment row — it belongs inline with the other per-assignment config fields (`max_input_tokens`, `rate_input`, etc.). A junction table would add a join to every proxy cache load for no benefit.

### 2. Skip condition placement in failover loop

The check is inserted after the `max_input_tokens` check and before the HTTP request is made. The order of skip conditions is:

1. No API key → skip
2. Circuit breaker open → skip
3. Rate limited → skip
4. Max input tokens exceeded → skip
5. **Supported models mismatch → skip** (new)
6. Attempt HTTP request

This placement ensures the model check is cheap (no I/O) and happens before any network cost.

### 3. model_mappings implicit support

If the requested model is a key in `model_mappings`, the server is considered to support it even if the model is not listed in `supported_models`. This avoids requiring admins to duplicate entries. The check uses the original request model (before mapping transform).

### 4. UpdateAssignment field type

`Option<Vec<String>>` is used (not `Option<Option<Vec<String>>>`). The field cannot be "nullable" in the SQL sense (it has a NOT NULL default of `{}`). `None` in the Rust struct means "omit from update", `Some(vec)` means "set to this value". An empty `Some(vec![])` clears the list.

### 5. Redis cache invalidation

No special handling needed. The existing PUT assignment handler already calls `cache::invalidate_group_config` after every update. The new field will be included in the cached struct automatically.

### 6. Frontend model source

The multi-select chips input is populated from the existing models store/API (same source used by other model-selection UI in the app). Model names are stored as free strings — the UI provides convenience but does not enforce referential integrity.

## Risks / Trade-offs

- [Cache struct change on deploy] Existing cached `GroupConfig` entries in Redis will not have `supported_models`. Deserialization uses `#[serde(default)]` so old cache entries deserialize with an empty vec (= no filtering). No cache flush needed on deploy.
- [All servers skipped] If every server in a group has a non-empty `supported_models` list that excludes the requested model, the existing "all servers exhausted" error path is returned. No new error surface, but admins should be aware of this configuration risk.
- [Free-text model names] Typos in `supported_models` will silently cause servers to be skipped for that model. The UI populating from the models table mitigates this for known models.

## Migration Plan

1. Deploy migration: `ALTER TABLE group_servers ADD COLUMN supported_models TEXT[] NOT NULL DEFAULT '{}'`
2. Deploy backend: new field in structs and queries, skip logic in proxy loop
3. Deploy frontend: new multi-select in edit dialog
4. No rollback complexity — column has a safe default; removing the column later is a standard migration

## Open Questions

None — all decisions resolved in planning.
