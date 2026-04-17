## Context

The proxy engine routes requests through a waterfall of servers within a group. When a server returns a status code listed in the group's `failover_status_codes`, the proxy immediately moves to the next server. There is no mechanism to retry the same server before failing over.

Transient errors (e.g., 503 Service Unavailable, 429 Too Many Requests) often resolve within seconds. Immediately failing over wastes the failover slot and can cause unnecessary load redistribution. Per-server retry config lets operators tune retry behavior independently for each server — a server behind a rate-limited API might retry on 429, while a server behind a flaky upstream might retry on 503.

The existing `group_servers` table already carries per-server nullable config fields (rate limit, active hours, max input tokens) using the all-or-nothing pattern. This change follows the same pattern.

## Goals / Non-Goals

**Goals:**
- Add three nullable columns to `group_servers`: `retry_status_codes`, `retry_count`, `retry_delay_seconds`
- Proxy retries the same server up to `retry_count` times with `retry_delay_seconds` delay when the response status is in `retry_status_codes`, before falling through to failover
- Admin API accepts and validates the three new fields on the update-assignment endpoint
- Frontend provides a per-server retry config dialog and a badge indicator when configured
- Retry fields are included in the Redis-cached `GroupServerDetail` automatically

**Non-Goals:**
- Group-level retry config (retry is per-server only)
- Exponential backoff or jitter (fixed delay only)
- Retry on connection errors (only on HTTP status codes)
- Retry budget or global retry limits across servers

## Decisions

**All-or-nothing validation (same as rate limit and active hours)**
Either all three fields are null (retry disabled) or all three are non-null (retry enabled). Partial config is rejected with HTTP 400. This avoids ambiguous states where retry is "half-configured".

**Retry before failover, not instead of failover**
After all retries are exhausted, the proxy falls through to the existing failover check. If the final retry response status is in `failover_status_codes`, failover proceeds normally. This preserves existing failover semantics and makes retry purely additive.

**`retry_status_codes` is separate from `failover_status_codes`**
Group-level `failover_status_codes` controls when to move to the next server. Per-server `retry_status_codes` controls when to retry the same server. They can overlap (e.g., retry 503 twice, then failover on 503) or be disjoint. The retry check happens first; failover check happens after retries are exhausted.

**`tokio::time::sleep` for delay**
Simple fixed delay using the async sleep primitive already available in the Tokio runtime. No additional dependencies needed.

**`#[serde(default)]` on new Rust fields**
New fields on `GroupServerDetail` must carry `#[serde(default)]` so that existing Redis-cached objects (serialized before this deployment) deserialize without error, defaulting to `None`.

**Double-Option pattern for `UpdateAssignment`**
Consistent with existing nullable fields: `Option<Option<Vec<i32>>>` for `retry_status_codes`, `Option<Option<i32>>` for `retry_count`, `Option<Option<f64>>` for `retry_delay_seconds`. Outer `None` = omitted (no change), inner `None` = explicit null (clear).

**UI: separate icon button + dialog (same as model mappings `tune` button)**
Retry config is complex enough (3 fields + a dynamic list of status codes) to warrant its own dialog rather than inline editing. The `replay` icon is semantically appropriate. A badge on the server row indicates when retry is active, consistent with the rate limit badge pattern.

## Risks / Trade-offs

**Increased latency on retried errors** → Mitigation: retry is opt-in per server; operators choose which status codes to retry and set appropriate delays. Default delay of 1.0s is conservative.

**Redis cache staleness after deployment** → Mitigation: `#[serde(default)]` on new fields ensures old cached objects deserialize cleanly with `None` values (retry disabled), which is the correct safe default.

**Retry loop adds complexity to proxy hot path** → Mitigation: the retry check is a simple `if let Some(retry_cfg) = ...` guard; when retry is not configured (the common case), the code path is unchanged.

**Status code overlap between retry and failover** → This is intentional and documented. Retry fires first; failover fires after retries are exhausted. Operators must understand this ordering when configuring both.

## Migration Plan

1. Deploy migration `040_group_servers_retry.sql` — adds three nullable columns with `DEFAULT NULL`; no existing rows are affected
2. Deploy backend — new fields are optional in all structs; existing cached Redis data deserializes safely via `#[serde(default)}`
3. Deploy frontend — new UI is additive; no existing functionality changes
4. Rollback: columns can be dropped without data loss (all NULL); backend rollback requires redeploying previous binary

## Open Questions

None — all decisions resolved in planning session.
