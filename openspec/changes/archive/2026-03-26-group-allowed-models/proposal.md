## Why

Groups currently pass through any model name to upstream servers without restriction. Administrators need to control which models each group (and sub-key) can use — enforcing cost control and access policies at the proxy layer before requests reach upstream providers.

## What Changes

- Add a `models` master table for reusable model name definitions across groups
- Add `group_allowed_models` junction table linking groups to permitted models
- Add `group_key_allowed_models` junction table for sub-key level model restrictions (subset of group's allowed models)
- Add admin CRUD API for the models master list (`/api/admin/models`)
- Add admin API for managing group allowed models (`/api/admin/groups/{id}/allowed-models`)
- Add admin API for managing key allowed models (`/api/admin/groups/{id}/keys/{key_id}/allowed-models`)
- Add proxy-layer model validation: block requests with disallowed models before forwarding upstream (HTTP 403 `permission_error`)
- Add "Allowed Models" tab in GroupDetailPage UI
- Add allowed models picker in sub-key management UI
- Extend `GroupConfig` cache with `allowed_models` and `key_allowed_models` fields

## Capabilities

### New Capabilities
- `model-allowlist`: Master model list management (CRUD) and per-group/per-key model restriction with proxy enforcement

### Modified Capabilities
- `group-management`: Groups gain allowed model configuration (new tab, new API endpoints, new cache fields)
- `proxy-engine`: Proxy gains model validation check before failover waterfall

## Impact

- **Database**: 3 new tables (`models`, `group_allowed_models`, `group_key_allowed_models`), new migration file
- **Backend API**: New route modules for models CRUD, group allowed models, key allowed models
- **Proxy**: New validation step in `proxy_handler` after `extract_request_model`, before failover loop
- **Cache**: `GroupConfig` struct extended, `resolve_group_config` query updated, existing invalidation pattern reused
- **Frontend**: New tab in GroupDetailPage, new store methods, allowed models picker in key management
