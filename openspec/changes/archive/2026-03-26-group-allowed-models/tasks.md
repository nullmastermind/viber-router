## 1. Database Migration

- [x] 1.1 Create migration file with `models` table (id UUID PK, name TEXT UNIQUE, created_at TIMESTAMPTZ)
- [x] 1.2 Add `group_allowed_models` junction table (group_id FK, model_id FK, created_at, PK(group_id, model_id), ON DELETE CASCADE on group_id, ON DELETE RESTRICT on model_id)
- [x] 1.3 Add `group_key_allowed_models` junction table (group_key_id FK, model_id FK, created_at, PK(group_key_id, model_id), ON DELETE CASCADE on group_key_id, ON DELETE RESTRICT on model_id) ← (verify: all 3 tables created, FK constraints correct, indexes on foreign keys)

## 2. Backend Models (Rust structs)

- [x] 2.1 Add `Model` struct (id, name, created_at) with `FromRow`, `Serialize`, `Deserialize` in `src/models/model.rs`
- [x] 2.2 Add `CreateModel` input struct and `ModelListItem` if needed
- [x] 2.3 Add `GroupAllowedModel` struct for the junction table
- [x] 2.4 Extend `GroupConfig` with `allowed_models: Vec<String>` and `key_allowed_models: Vec<String>` fields
- [x] 2.5 Register new model module in `src/models/mod.rs` ← (verify: all structs compile, GroupConfig fields added)

## 3. Models Master List API

- [x] 3.1 Create `src/routes/admin/models.rs` with router function
- [x] 3.2 Implement `list_models` handler (GET, paginated, name search via ILIKE)
- [x] 3.3 Implement `create_model` handler (POST, unique name check → 409 on duplicate)
- [x] 3.4 Implement `delete_model` handler (DELETE, check group usage → 409 if in use, else 204)
- [x] 3.5 Register models router in admin routes ← (verify: all CRUD endpoints respond correctly, 409 on duplicate/in-use)

## 4. Group Allowed Models API

- [x] 4.1 Create `src/routes/admin/group_allowed_models.rs` with router function
- [x] 4.2 Implement `list_group_allowed_models` handler (GET, returns model objects for group)
- [x] 4.3 Implement `add_group_allowed_model` handler (POST, supports both `model_id` and `name` input, creates model if needed, 409 on duplicate assignment)
- [x] 4.4 Implement `remove_group_allowed_model` handler (DELETE, cascade-delete from sub-keys, invalidate cache, 204)
- [x] 4.5 Register group_allowed_models router nested under groups (`/{group_id}/allowed-models`) ← (verify: assign/remove works, cascade to sub-keys on remove, cache invalidated)

## 5. Key Allowed Models API

- [x] 5.1 Create `src/routes/admin/group_key_allowed_models.rs` with router function
- [x] 5.2 Implement `list_key_allowed_models` handler (GET, returns model objects for key)
- [x] 5.3 Implement `add_key_allowed_model` handler (POST, validate model is in group's allowed list, validate group has allowed models configured, 400 if not, 409 on duplicate)
- [x] 5.4 Implement `remove_key_allowed_model` handler (DELETE, invalidate cache, 204)
- [x] 5.5 Register group_key_allowed_models router nested under group keys (`/{group_id}/keys/{key_id}/allowed-models`) ← (verify: subset constraint enforced, 400 when group has no allowed models)

## 6. Proxy Model Validation

- [x] 6.1 Update `resolve_group_config` in `proxy.rs` to query `group_allowed_models` + `models` and populate `allowed_models` field
- [x] 6.2 Update `resolve_group_config` to query `group_key_allowed_models` + `models` and populate `key_allowed_models` field (when request uses sub-key)
- [x] 6.3 Add model validation check in `proxy_handler` after `extract_request_model`, before failover loop: if `allowed_models` non-empty → check model, then if `key_allowed_models` non-empty → check model. Return 403 `permission_error` on failure.
- [x] 6.4 Handle missing `model` field: if `allowed_models` non-empty and no model in request → 403 ← (verify: disallowed model returns 403, allowed model passes through, empty list = pass-through, key-level restriction works, no-model-field blocked)

## 7. Group Detail Response

- [x] 7.1 Update `get_group` admin handler to include `allowed_models` array in the response (join `group_allowed_models` + `models`)
- [x] 7.2 Update `GroupWithServers` or create extended response struct to include allowed models ← (verify: GET /api/admin/groups/{id} returns allowed_models field)

## 8. Frontend — Models Store & Types

- [x] 8.1 Add `Model` interface to types (id, name, created_at)
- [x] 8.2 Create `src/stores/models.ts` Pinia store with: fetchModels (paginated, search), createModel, deleteModel
- [x] 8.3 Add allowed models methods to groups store: fetchGroupAllowedModels, addGroupAllowedModel, removeGroupAllowedModel
- [x] 8.4 Add key allowed models methods to groups store: fetchKeyAllowedModels, addKeyAllowedModel, removeKeyAllowedModel ← (verify: all store methods call correct API endpoints)

## 9. Frontend — Allowed Models Tab

- [x] 9.1 Add "Allowed Models" tab to GroupDetailPage (between Keys and TTFT tabs)
- [x] 9.2 Build allowed models list view with model name and remove button
- [x] 9.3 Build model picker: autocomplete/combobox from master models list with option to create new model inline
- [x] 9.4 Wire up add/remove actions with cache refresh (reload group after mutation) ← (verify: tab shows allowed models, can add from master list, can create new inline, can remove, list updates after mutations)

## 10. Frontend — Key Allowed Models

- [x] 10.1 Add allowed models section to key management UI (only visible when group has allowed models)
- [x] 10.2 Build key model picker: select from group's allowed models list only
- [x] 10.3 Wire up add/remove actions for key allowed models ← (verify: picker only shows group's allowed models, add/remove works, hidden when group has no allowed models)

## 11. Lint & Type Check

- [x] 11.1 Run `just check` and fix all errors ← (verify: `just check` passes with zero errors)
