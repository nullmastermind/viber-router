## Why

The system tracks token usage (input, output, cache write, cache read) per request but has no way to translate token counts into USD cost. Administrators need to see estimated costs alongside token counts to understand spending, and the data model must support future budget-limit enforcement per group key.

## What Changes

- Add per-model pricing fields (USD per 1M tokens) for input, output, cache write, and cache read token types to the `models` table.
- Add per-group-server rate multiplier fields (default 1.0) for each token type to the `group_servers` table, allowing cost markup/discount per provider within a group.
- Extend the token usage stats API to compute and return cost per row using the formula: `cost = tokens × model_price × server_rate / 1_000_000`.
- Create a dedicated Models management page with pricing configuration UI.
- Add a rate multiplier tag (`[x1.0]`) to each server row in the group detail servers tab, with a click-to-edit modal.
- Add a "Cost ($)" column to the token usage table and sub-key usage component, with a total row at the bottom. Display "—" when a model has no pricing configured.

## Capabilities

### New Capabilities
- `model-pricing`: CRUD for per-model token pricing (4 USD/1MTok fields on the models table), exposed via API and a new Models admin page.
- `server-cost-rate`: Per-group-server rate multiplier fields (4 rates, default 1.0) on `group_servers`, editable via a modal in the group detail servers tab.
- `token-usage-cost`: Server-side cost calculation joining model pricing and server rates with token usage logs, returned in the existing token usage stats API response and displayed in the UI.

### Modified Capabilities
- `token-usage-stats-api`: Response schema changes to include cost fields per row and a total cost summary.
- `token-usage-ui`: Token usage table gains a "Cost ($)" column and a total row.

## Impact

- **Database**: Two migrations — ALTER `models` (4 nullable NUMERIC columns), ALTER `group_servers` (4 nullable FLOAT8 columns).
- **Backend API**: `PUT /api/admin/models/:id` gains pricing fields. `PUT /api/admin/groups/:id/servers/:sid` gains rate fields. `GET /api/admin/token-usage` response adds cost fields.
- **Frontend**: New `/models` route and page. Modified `GroupDetailPage.vue` (servers tab rate tag + modal, token usage tab cost column). Modified `SubKeyUsage.vue` (cost column). Updated stores (`models.ts`, `groups.ts`).
- **No proxy impact**: Pricing and rates are admin-display-only; they are not part of `GroupConfig` or the Redis cache.
