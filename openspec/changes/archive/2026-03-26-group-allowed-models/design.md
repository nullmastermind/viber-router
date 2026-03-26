## Context

Viber Router is a proxy that routes API requests to upstream LLM providers. Groups define routing rules (server priority, model mappings, failover). Each group has a master API key and optional sub-keys (`group_keys`) with independent rate limits.

Currently, any model name in the request body is passed through to upstream servers (with optional per-server model name mapping). There is no mechanism to restrict which models a group or sub-key can use.

The existing data model follows a pattern of master tables + junction tables: `servers` + `group_servers`, `groups` + `group_keys`. Cache is stored in Redis keyed by API key, invalidated on any admin mutation via `invalidate_group_all_keys`.

## Goals / Non-Goals

**Goals:**
- Allow admins to define a master list of model names (reusable across groups)
- Allow admins to restrict which models a group can use (allowlist)
- Allow admins to further restrict which models a sub-key can use (subset of group's allowlist)
- Block disallowed model requests at the proxy layer before any upstream call
- Maintain backward compatibility: empty allowlist = pass-through (current behavior)

**Non-Goals:**
- Model-level rate limiting or quota per model
- Automatic model discovery from upstream providers
- Model aliasing or renaming (existing `model_mappings` on `group_servers` handles this)
- Wildcard or pattern matching for model names (exact match only)

## Decisions

### 1. Separate `models` master table + junction tables

**Decision**: Three new tables — `models` (master list), `group_allowed_models` (group junction), `group_key_allowed_models` (sub-key junction).

**Rationale**: Follows the established `servers` + `group_servers` pattern. A master table enables reuse across groups — admins pick from a dropdown instead of retyping model names. This is critical UX when managing many groups.

**Alternative considered**: JSONB array column on `groups` table (like `failover_status_codes`). Rejected because it requires retyping model names per group and doesn't support a shared picker UI.

### 2. Two-layer permission model (group → key)

**Decision**: Group-level allowlist checked first, then key-level allowlist (subset of group's list).

```
Group allowed_models: [sonnet, opus, gpt-4o]
  Key A allowed_models: [sonnet]         → only sonnet
  Key B allowed_models: [sonnet, opus]   → sonnet + opus
  Key C allowed_models: []               → inherits group → all 3
```

**Constraints**:
- Key can only select models from its parent group's allowed list
- Key-level config is only available when group has a non-empty allowed list
- Empty key allowlist = inherit all group-allowed models

**Rationale**: Provides granular control without complexity. Sub-keys are often distributed to different teams/users who should only access specific models.

### 3. Proxy validation placement

**Decision**: Check allowed models in `proxy_handler` after `extract_request_model` and before the failover waterfall loop.

**Rationale**: Fail fast — reject disallowed requests before any upstream network call. The model name is already extracted at this point. Both `allowed_models` and `key_allowed_models` are available in the cached `GroupConfig`.

### 4. Error response format

**Decision**: HTTP 403 with Anthropic-compatible error format:
```json
{
  "type": "error",
  "error": {
    "type": "permission_error",
    "message": "Your API key does not have permission to use the specified model."
  }
}
```

**Rationale**: Matches Anthropic's own response when a key lacks model permission. Clients already handle this error type.

### 5. Cascade behavior on group model removal

**Decision**: When a model is removed from a group's allowed list, cascade-delete it from all sub-keys of that group.

**Rationale**: Maintains the invariant that key allowed models are always a subset of group allowed models. Simpler than requiring admins to manually clean up sub-keys first.

### 6. Model deletion protection

**Decision**: Deleting a model from the master list is blocked (HTTP 409) if any group references it. Admin must remove it from all groups first.

**Rationale**: Follows the same pattern as server deletion — prevents accidental removal of in-use models.

### 7. Requests without a model field

**Decision**: If a group has a non-empty allowed models list and the request body has no `model` field, return 403.

**Rationale**: A request without a model field to a model-restricted group is likely malformed. Blocking it prevents unexpected behavior at the upstream provider.

## Risks / Trade-offs

- **Cache size increase**: `GroupConfig` grows by two `Vec<String>` fields. Minimal impact — model names are short strings, typical lists are <20 entries. → No mitigation needed.
- **Cascade delete on group model removal**: Could surprise admins if sub-keys silently lose model access. → The UI should show which keys are affected before confirming removal (future enhancement, not in scope).
- **No model validation against upstream**: The master model list is admin-managed strings — no verification that a model name actually exists at any provider. → Acceptable for an admin tool; admins know their model names.
