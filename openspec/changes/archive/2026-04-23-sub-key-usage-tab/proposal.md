## Why

Admins currently have no way to see aggregated token usage broken down by sub-key within a group. The existing Token Usage tab only shows usage per server/model. When a group has many sub-keys (potentially hundreds), admins must click into each key individually to view its usage. A dedicated "By Sub-Key" view provides a single-screen summary of which keys are consuming resources and at what cost.

## What Changes

- Add a new backend endpoint `GET /api/admin/token-usage/by-key` that aggregates token usage per `group_key_id`, joining `group_keys` to include key name and creation date.
- Split the existing Token Usage tab on the Group Detail page into two child tabs: "By Server" (current behavior, unchanged) and "By Sub-Key" (new).
- The "By Sub-Key" child tab displays a table of all sub-keys with their aggregated usage, supports date-range filtering, column sorting, and expandable rows showing each key's subscriptions.

## Capabilities

### New Capabilities
- `token-usage-by-key-api`: Backend endpoint that returns token usage aggregated per group_key_id, with cost calculation, supporting period shortcuts and absolute date ranges.
- `token-usage-by-key-ui`: Frontend child tab within Token Usage showing per-sub-key usage table with date filtering, sorting, expandable subscription rows, and totals.

### Modified Capabilities
- `token-usage-ui`: The Token Usage section on the Group Detail page is restructured to use child tabs ("By Server" | "By Sub-Key") instead of rendering the server table directly.

## Impact

- **Backend**: New route added under `/api/admin/token-usage/by-key` in `viber-router-api/src/routes/admin/`. New handler registered in `admin/mod.rs`. Reuses existing `resolve_interval`, cost-calculation SQL pattern, and `AppState`.
- **Frontend**: `GroupDetailPage.vue` token-usage tab panel refactored to include `q-tabs`/`q-tab-panels` for child navigation. New store action and interface added in `src/stores/groups.ts`. Existing "By Server" content moves into a child tab with zero behavior change.
- **Database**: No schema changes. Queries against existing `token_usage_logs` and `group_keys` tables.
