## Context

The proxy's failover waterfall currently skips servers only for circuit breaker state and rate limiting. There is no mechanism to route based on the estimated size of the request's input tokens. Operators who use per-token-priced servers alongside flat-rate servers have no way to protect the former from unexpectedly large (and therefore expensive) requests.

The `group_servers` table already carries per-assignment configuration fields (`cb_*`, `max_requests`, `rate_window_seconds`, `normalize_cache_read`). Adding `max_input_tokens` follows the same established pattern: a nullable column that defaults to NULL for backward compatibility, surfaced through the same model structs, admin API, and frontend edit dialog.

## Goals / Non-Goals

**Goals:**

- Add `max_input_tokens INTEGER NULL` to `group_servers` via a new migration
- Estimate input tokens from the request body before the failover loop begins (one computation per request, not per server)
- Skip servers during the failover loop when estimated tokens exceed the server's threshold
- Expose the field through all existing layers: Rust models, admin API, frontend store, and edit dialog
- Display a badge in the server list when a threshold is configured

**Non-Goals:**

- Accurate token counting (tiktoken, Anthropic tokenizer, etc.) — approximate estimation is sufficient
- Tracking or logging which servers were skipped due to this threshold
- Applying the threshold to non-message endpoints (count_tokens, etc.)
- Global per-server threshold (only per-server-in-group is needed)

## Decisions

### Decision: Estimate tokens before the loop, not inside it

Token estimation reads and parses the request body once. Computing it inside the loop per-server would be wasteful. The body bytes are already buffered by the time the proxy enters the waterfall loop. Estimation is done once immediately before the loop, stored as `Option<usize>`, and compared against each server's `max_input_tokens` inside the loop.

Alternatives considered:
- Per-server lazy computation: rejected because the body is already available upfront and parsing multiple times is unnecessary.
- Using a real tokenizer: rejected because it adds a heavy dependency and compile complexity; len/4 after stripping image blocks is a well-known approximation used elsewhere.

### Decision: Image content blocks are stripped before estimation

Base64-encoded image data dominates JSON byte length and is irrelevant to text token cost. Stripping `{"type": "image", ...}` blocks before measuring gives a far more meaningful estimate of text token volume.

Implementation approach:
1. Parse the request body as `serde_json::Value`
2. If the top-level object has a `messages` array, filter each message's `content` array to remove objects where `"type" == "image"`
3. Serialize back to string
4. `estimated_tokens = filtered_string.len() / 4`

If JSON parsing fails (e.g., non-JSON body), estimation is skipped and all servers are attempted (fail open).

Alternatives considered:
- Regex-based stripping: fragile, rejected.
- Counting only the `messages` field: same result but more complex extraction.

### Decision: Skip behavior — skip server, continue waterfall

A server whose threshold is exceeded is silently skipped, matching the circuit breaker and rate limit skip patterns. No special error is returned when a server is skipped. If ALL servers are skipped due to token threshold (and none are skipped for other reasons), the final "no servers available" error from the existing exhaustion path is returned rather than a new bespoke error. This keeps the behavior consistent and avoids over-engineering the error taxonomy.

Alternatives considered:
- Return a distinct HTTP error when all servers are token-skipped: considered, but adds complexity and edge-case handling without meaningful operator benefit for now.

### Decision: `max_input_tokens` is `Option<i32>` in Rust, `number | null` in TypeScript

Consistent with all other nullable integer fields in the codebase (`cb_max_failures`, `max_requests`, etc.). The double-Option pattern `Option<Option<i32>>` in `UpdateAssignment` (outer = not provided, inner = null to clear) is already used by every other nullable field and is the correct pattern here.

### Decision: Migration is additive only

`ALTER TABLE group_servers ADD COLUMN max_input_tokens INTEGER NULL` with no backfill. All existing rows get NULL automatically, preserving current behavior.

## Risks / Trade-offs

- **Estimation inaccuracy**: len/4 is approximate. A request that is 30,001 characters may be 7,000 actual tokens or 8,500 depending on content. Operators should set thresholds with a safety margin. → Mitigation: document in UI label that the value is an approximate upper bound.
- **JSON parse failure fails open**: If the body is not valid JSON, estimation returns None and all servers are attempted. This is the safe default but means the threshold is ineffective for malformed requests. → Acceptable: malformed requests will fail at the upstream anyway.
- **No skip logging**: Skipped-due-to-token-threshold servers are invisible in logs beyond the final failover chain. → Acceptable for v1; can be added later.

## Migration Plan

1. Write and apply migration `031_add_max_input_tokens.sql`
2. Deploy backend (all NULL defaults, backward compatible)
3. Deploy frontend (new field in edit dialog)
4. Operators configure thresholds via admin UI as needed

Rollback: `ALTER TABLE group_servers DROP COLUMN max_input_tokens` removes the column. Backend must be rolled back first so it does not reference the dropped column.

## Open Questions

None. All design decisions are resolved.
