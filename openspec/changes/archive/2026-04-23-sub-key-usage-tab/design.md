## Context

The Group Detail page currently shows a "Token Usage" tab with a single table: usage aggregated by server and model. The backend endpoint `GET /api/admin/token-usage` supports filtering by `group_key_id` for individual keys, but there is no endpoint that aggregates across all sub-keys in a single query.

Groups can have hundreds of sub-keys. Without a dedicated aggregation endpoint, displaying per-key usage would require N+1 API calls (one per key), which is impractical. The `token_usage_logs` table already stores `group_key_id` (nullable UUID, FK to `group_keys`), so the data is available for aggregation.

The frontend `GroupDetailPage.vue` has an existing pattern for expandable rows with subscriptions (used in the Keys tab) that can be reused for the new sub-key usage rows.

## Goals / Non-Goals

**Goals:**
- Provide a single-request backend endpoint that returns usage aggregated per sub-key, with cost calculation.
- Add a "By Sub-Key" child tab within the existing Token Usage tab, preserving the current "By Server" view as the default child tab.
- Support date-range filtering, column sorting, and expandable subscription details on each key row.

**Non-Goals:**
- Charting or visualization of per-key usage trends (future work).
- Modifying the existing "By Server" aggregation endpoint or its behavior.
- Adding export functionality for the sub-key usage data.
- Real-time or WebSocket-based usage updates.

## Decisions

### 1. New endpoint vs. extending existing endpoint

**Decision**: Create a new endpoint `GET /api/admin/token-usage/by-key` rather than adding an `aggregation=by-key` parameter to the existing endpoint.

**Rationale**: The existing endpoint returns `ServerTokenUsage` structs grouped by server/model. The new endpoint returns a fundamentally different shape (`KeyTokenUsage` grouped by `group_key_id`). Separate endpoints keep response types clear and avoid conditional response shapes. The route nests naturally under the existing `/api/admin/token-usage` prefix.

**Alternative considered**: Adding `?group_by=key` to existing endpoint. Rejected because it would require a union response type or dynamic field selection, complicating both backend serialization and frontend type safety.

### 2. Child tabs vs. separate top-level tab

**Decision**: Use child tabs within the existing "Token Usage" tab panel rather than adding a new top-level tab.

**Rationale**: Both views are about token usage — grouping them together keeps the tab bar from growing and maintains logical grouping. Child tabs are a standard Quasar pattern (nested `q-tabs` + `q-tab-panels`). The "By Server" tab remains the default, so existing user workflows are undisrupted.

### 3. Date filtering approach for "By Sub-Key"

**Decision**: Use start/end date pickers (defaulting to 30 days ago through now) rather than the period-button toggle used in "By Server".

**Rationale**: The plan specifies date pickers for this tab. The backend still supports period shortcuts for internal/programmatic use, but the UI surfaces explicit date pickers for flexibility. This gives admins precise control over the reporting window for sub-key analysis.

### 4. NULL group_key_id handling

**Decision**: Rows where `group_key_id IS NULL` (master key / dynamic key usage) are returned as a single aggregated row with `key_name` set to "Master / Dynamic Keys" and `group_key_id` set to null in the JSON response.

**Rationale**: Excluding NULL rows would hide usage data. Presenting them as a labeled summary row gives complete coverage without requiring schema changes.

### 5. Cost calculation approach

**Decision**: Reuse the same cost-calculation SQL pattern from the existing `get_token_usage` handler (JOIN models + group_servers for rates, compute cost per million tokens).

**Rationale**: Consistency with existing cost figures. The SQL is already proven and handles edge cases (NULL pricing, rate multipliers). The only difference is the GROUP BY clause changes from `server_id, model` to `group_key_id`.

## Risks / Trade-offs

- **[Performance with large groups]** Groups with 500+ sub-keys and high usage volume may produce slow aggregation queries. Mitigation: the `token_usage_logs` table is partitioned by `created_at`, and an existing composite index on `(group_id, created_at)` supports the query. The date range filter bounds the partition scan.
- **[NULL key_name for deleted keys]** If a `group_key` is deleted, the LEFT JOIN will produce NULL for `key_name`. Mitigation: the frontend will display "Deleted Key" as a fallback label; this is informational only.
- **[Child tab state persistence]** Switching between "By Server" and "By Sub-Key" tabs will trigger re-fetches if data is not cached. Mitigation: acceptable for an admin tool; each tab manages its own loading state independently.
