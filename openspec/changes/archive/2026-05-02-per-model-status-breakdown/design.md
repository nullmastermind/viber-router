## Context

The public usage page (`/#/usage`) displays a single overall uptime status with 90 x 30-minute uptime bars, sourced from the `uptime_checks` table. This table tracks chain-level health checks and has no `request_model` column. However, the `proxy_logs` table stores every proxied request with `request_model`, `status_code`, `group_id`, and `created_at` -- providing sufficient data to derive per-model uptime. The group's allowed models are stored in `group_allowed_models` joined to `models`.

The frontend already has a reusable `UptimeBars` component and helper functions (`statusBadgeColor`, `statusBadgeLabel`) that can be applied to each model row.

## Goals / Non-Goals

**Goals:**
- Show per-model uptime status alongside the existing overall status on the public usage page.
- Derive per-model status from actual proxy traffic (`proxy_logs`) rather than synthetic health checks.
- Display all models from the group's allowed list, including those with no recent traffic (shown as "no_data").

**Non-Goals:**
- Replacing the existing overall status (it stays as-is, sourced from `uptime_checks`).
- Adding per-model status to the admin uptime page (this is strictly for the public page).
- Creating new database tables or migrations.
- Adding per-model historical charts or detailed analytics beyond the 90-bucket bars.

## Decisions

### 1. Data source: `proxy_logs` for per-model, `uptime_checks` for overall

The overall status continues using `uptime_checks` (chain-level health probes). Per-model status uses `proxy_logs` because it has `request_model`. This means the two status views may occasionally disagree (e.g., health check fails but actual traffic succeeds), which is acceptable since they measure different things.

**Alternative considered**: Adding `request_model` to `uptime_checks`. Rejected because uptime checks are synthetic probes, not real traffic, so they cannot meaningfully represent per-model status.

### 2. Per-model success definition: individual request status codes

For per-model buckets, each row in `proxy_logs` is evaluated independently -- a request is successful if its `status_code` is 200-299. This differs from the overall status which groups by `request_id` and uses `bool_or(status_code BETWEEN 200 AND 299)` across retry chains.

**Rationale**: Per-model status shows each model's own success rate. The chain-level retry aggregation is specific to the overall uptime check model and does not apply to per-model breakdowns where each log row represents one attempt to one model.

### 3. Single endpoint extension (additive response)

The existing `GET /api/public/uptime` response is extended with a `models` array rather than creating a separate endpoint. This keeps the API surface minimal and ensures per-model data is fetched in a single request alongside overall status.

**Alternative considered**: Separate `GET /api/public/uptime/models` endpoint. Rejected to avoid an extra HTTP round-trip and to keep the frontend code simpler.

### 4. Model list from `group_allowed_models`

The allowed model names are fetched by joining `group_allowed_models` and `models` using the `group_id` from the key lookup. This ensures only configured models appear, not arbitrary strings from `proxy_logs.request_model`. Models with no traffic get empty buckets and `no_data` status.

### 5. Frontend: always-visible per-model section

Per-model rows are always visible below the overall status (not collapsible, no toggle). This keeps the UI simple and makes model-level issues immediately visible.

## Risks / Trade-offs

**[Performance] Query against proxy_logs for 45 hours of data** -- `proxy_logs` is a partitioned table and can be large. The query groups by `request_model` and time bucket. Mitigation: The existing index on `(group_id)` plus partition pruning on `created_at` should keep this performant. The 45-hour window limits the scan range. Monitor query time after deployment.

**[Inconsistency] Overall vs per-model status may disagree** -- Since they use different data sources and different success definitions, a model could show "operational" while overall shows "degraded" (or vice versa). Mitigation: This is by design -- they measure different things. The overall status reflects health check probes; per-model reflects actual proxy traffic. Both are valuable signals.

**[NULL request_model] Some proxy_logs rows may have NULL request_model** -- These are excluded from per-model aggregation (they do not match any allowed model name). This is acceptable since the overall status already covers them.
