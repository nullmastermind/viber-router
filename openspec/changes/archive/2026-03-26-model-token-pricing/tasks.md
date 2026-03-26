## 1. Database Migrations

- [x] 1.1 Create migration to add pricing columns to `models` table: `input_1m_usd NUMERIC`, `output_1m_usd NUMERIC`, `cache_write_1m_usd NUMERIC`, `cache_read_1m_usd NUMERIC` (all nullable)
- [x] 1.2 Create migration to add rate columns to `group_servers` table: `rate_input FLOAT8`, `rate_output FLOAT8`, `rate_cache_write FLOAT8`, `rate_cache_read FLOAT8` (all nullable) ← (verify: both migrations run cleanly, columns exist with correct types and nullability)

## 2. Backend — Model Pricing API

- [x] 2.1 Add pricing fields to `Model` struct in `src/models/model.rs`: `input_1m_usd`, `output_1m_usd`, `cache_write_1m_usd`, `cache_read_1m_usd` (all `Option<f64>`)
- [x] 2.2 Create `UpdateModel` struct with optional name and 4 optional pricing fields
- [x] 2.3 Add `PUT /api/admin/models/:id` handler in `src/routes/admin/models.rs` — validate non-negative pricing, update model, return updated record
- [x] 2.4 Update `list_models` and `create_model` to include pricing fields in SELECT and INSERT ← (verify: GET returns pricing fields, PUT validates negatives with 400, create includes pricing)

## 3. Backend — Server Cost Rate API

- [x] 3.1 Add rate fields to `GroupServer` and `GroupServerDetail` structs — NO: rate fields are NOT added to `GroupServerDetail` (proxy struct). Add only to `GroupServer` struct and a new response type for admin
- [x] 3.2 Add rate fields to `UpdateAssignment` struct: `rate_input`, `rate_output`, `rate_cache_write`, `rate_cache_read` (all `Option<Option<f64>>`)
- [x] 3.3 Update `update_assignment` handler in `src/routes/admin/group_servers.rs` to persist rate fields with non-negative validation
- [x] 3.4 Ensure rate fields are returned in the group detail server list (admin GET endpoint) ← (verify: rates persist correctly, negative values rejected with 400, rates returned in admin response)

## 4. Backend — Token Usage Cost Calculation

- [x] 4.1 Add `cost_usd` field (`Option<f64>`) to `ServerTokenUsage` response struct in `src/routes/admin/token_usage.rs`
- [x] 4.2 Update token usage SQL queries (both absolute and relative period) to LEFT JOIN `models` ON `token_usage_logs.model = models.name` and LEFT JOIN `group_servers` ON `(token_usage_logs.group_id, token_usage_logs.server_id)`, computing cost using formula: `SUM(tokens * price * COALESCE(rate, 1.0)) / 1000000`
- [x] 4.3 Add unit tests for cost calculation edge cases: no pricing (NULL), zero tokens, zero rate, custom rate, partial pricing ← (verify: cost_usd is correct for all scenarios in spec, NULL when no pricing, LEFT JOINs handle orphaned rows)

## 5. Frontend — Models Page

- [x] 5.1 Add `/models` route to `src/router/routes.ts`
- [x] 5.2 Add Models sidebar link in `src/layouts/MainLayout.vue`
- [x] 5.3 Update `src/stores/models.ts` — add `updateModel` action with pricing fields, update `Model` type with pricing fields
- [x] 5.4 Create `src/pages/ModelsPage.vue` — table with name + 4 pricing columns, create/edit dialog with pricing fields, loading/error/empty states, "—" for NULL pricing
- [x] 5.5 Add non-negative validation on pricing inputs in the edit dialog ← (verify: Models page loads, CRUD works, pricing displays correctly, "—" for NULL, negative values prevented)

## 6. Frontend — Server Rate Tag & Modal

- [x] 6.1 Update `ServerTokenUsage` and related types in `src/stores/groups.ts` to include rate fields and `cost_usd`
- [x] 6.2 Add rate fields to the `updateAssignment` API call in `src/stores/groups.ts`
- [x] 6.3 Add clickable `[x1.0]` badge to each server row in `src/pages/GroupDetailPage.vue` servers tab (before the enable/disable toggle)
- [x] 6.4 Create rate edit modal in `GroupDetailPage.vue` — 4 rate inputs (nullable, min 0, placeholder "1.0"), save calls `updateAssignment` ← (verify: badge shows correct rate, modal opens/saves, rates persist after save, non-default rates reflected in badge)

## 7. Frontend — Token Usage Cost Display

- [x] 7.1 Add "Cost ($)" column to `tokenUsageColumns` in `GroupDetailPage.vue` — format as USD or "—"
- [x] 7.2 Add total row at bottom of token usage table showing sum of all cost values
- [x] 7.3 Add "Cost ($)" column to `SubKeyUsage.vue` with same formatting
- [x] 7.4 Run `just check` and fix any type/lint errors ← (verify: cost column shows correct values, total row sums correctly, "—" for no pricing, SubKeyUsage matches, `just check` passes clean)
