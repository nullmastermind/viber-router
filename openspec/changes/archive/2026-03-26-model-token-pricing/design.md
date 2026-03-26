## Context

Viber Router tracks token usage (input, output, cache write, cache read) per proxy request in `token_usage_logs`. The admin UI displays raw token counts but has no cost estimation. Anthropic and other LLM providers price tokens per 1M tokens with different rates for each token type and model. Administrators need cost visibility to understand spending.

Current state:
- `models` table: `id`, `name`, `created_at` — no pricing fields.
- `group_servers` table: assignment with priority, model_mappings, circuit breaker — no rate multiplier fields.
- `token_usage_logs.model` stores the request model name (pre-mapping).
- Token usage API returns raw token counts only.
- No dedicated Models admin page exists; models are managed inline in group detail.

## Goals / Non-Goals

**Goals:**
- Per-model pricing configuration (USD/1MTok) for 4 token types.
- Per-group-server rate multipliers (default 1.0) for cost markup/discount per provider.
- Server-side cost calculation in the token usage stats API.
- Dedicated Models page with pricing management.
- Cost column in token usage tables with total row.
- Data model supports future budget-limit enforcement (nullable pricing, rate fields).

**Non-Goals:**
- Budget limit enforcement (future work — data model supports it, no logic implemented).
- Real-time cost tracking in the proxy path (cost is admin-display-only).
- Proxy cache changes (pricing/rates are not part of GroupConfig).
- Model name aliasing or fuzzy matching (exact match only).

## Decisions

### D1: Pricing on `models` table, not a separate pricing table
Store 4 nullable NUMERIC columns directly on `models`. Simpler than a join table, and each model has exactly one price set. Nullable means "not configured" → cost displays as "—".

Alternative considered: Separate `model_pricing` table with FK to models. Rejected — adds complexity for a 1:1 relationship with no versioning requirement.

### D2: Rate multipliers on `group_servers`, not on `servers`
Rates are per-group-server assignment, not global per server. The same server in different groups can have different rate multipliers. This matches the user's requirement that rates are group-scoped.

### D3: Cost calculated server-side in the token usage query
JOIN `models` and `group_servers` in the SQL query to compute cost. This avoids sending pricing data to the frontend and keeps the calculation authoritative.

The query uses LEFT JOIN for both `models` (on `token_usage_logs.model = models.name`) and `group_servers` (on `token_usage_logs.group_id + server_id`). NULL model pricing → cost = NULL (displayed as "—"). NULL rates → treated as 1.0 via COALESCE.

### D4: Model matching by exact name on request model (pre-mapping)
`token_usage_logs.model` stores the request model name before `transform_model` applies `model_mappings`. Cost is calculated using this stored name matched against `models.name`. This is correct because:
- The request model is what the user/client sends (e.g., "claude-opus-4-6").
- Anthropic prices by this canonical model name.
- No reverse-lookup of mappings needed.

### D5: Rate tag UI — clickable badge with modal
Each server row in the group detail servers tab shows a `[x1.0]` badge (or the actual rate if non-default). Clicking opens a modal with 4 rate input fields. This keeps the server list clean while providing easy access to rate configuration.

### D6: No cache invalidation needed
Model pricing and server rates are only used in the admin token usage query. They are not part of `GroupConfig` or the Redis proxy cache. No invalidation logic needed.

## Risks / Trade-offs

- [Orphaned usage rows] Token usage logs reference `server_id` without FK constraint. If a server is removed from a group, LEFT JOIN on `group_servers` returns NULL rates → COALESCE to 1.0. Rows remain visible. → Acceptable behavior; matches existing INNER JOIN on `servers` table.
- [Model name mismatch] If `token_usage_logs.model` doesn't match any `models.name`, cost is NULL/"—". → Expected behavior per user decision. Admin can create the model with correct name.
- [No test harness for integration tests] Only inline `#[cfg(test)]` unit tests exist. → Scope to unit tests for cost calculation logic. Integration test infrastructure is out of scope.
