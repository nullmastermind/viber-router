## Why

The public usage page currently shows a single overall status (Operational/Degraded/Down) and one set of uptime bars for the entire group. This provides no visibility into which specific models are experiencing issues. When a single model degrades, the overall status may still show "Operational" while users of that model are impacted. Per-model status breakdown lets users quickly identify which models are healthy and which are having problems.

## What Changes

- Extend `GET /api/public/uptime` response to include a `models` array with per-model status, uptime percentage, and 90x30-min buckets.
- The per-model data is sourced from `proxy_logs` (which has `request_model`) instead of `uptime_checks` (which does not).
- Only models in the group's `allowed_models` list are included. Models with no traffic in the window get status `no_data`.
- The frontend renders a per-model status section below the existing overall status, showing each model's name, status badge, and `UptimeBars` component.
- The existing overall status section remains unchanged.

## Capabilities

### New Capabilities
- `per-model-uptime-api`: Extend the public uptime API response to include per-model status data derived from proxy_logs.
- `per-model-uptime-ui`: Render per-model status rows (name + badge + uptime bars) below the existing overall status on the public usage page.

### Modified Capabilities
- `uptime-public-api`: The response shape gains a new `models` array field alongside the existing fields.

## Impact

- **Backend**: `viber-router-api/src/routes/public/uptime.rs` -- extend response struct and add a second SQL query against `proxy_logs` grouped by `request_model`, plus a query to `group_allowed_models`/`models` to get the allowed model list.
- **Frontend**: `src/pages/PublicUsagePage.vue` -- add template section and TypeScript interface for per-model data; reuse existing `UptimeBars` component and `statusBadgeColor`/`statusBadgeLabel` helpers.
- **Database**: No schema changes or migrations required. All data comes from existing `proxy_logs`, `group_allowed_models`, and `models` tables.
- **API contract**: Additive change only -- existing response fields are preserved; a new `models` array is added.
