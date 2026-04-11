## Context

Viber Router proxies LLM API requests for groups of users. Currently, access control is at the group level (is_active) and model level (allowed_models), but there is no way to block specific clients identified by their User-Agent header. Admins have no visibility into which clients are accessing a group, and revoking an API key is the only recourse when a specific client needs to be blocked.

The system already has patterns for per-group config fields (allowed_models, blocked_api_paths), Redis-backed caching of GroupConfig, and admin CRUD routes nested under groups. This change follows those patterns closely.

## Goals / Non-Goals

**Goals:**
- Record unique user-agents seen per group with minimal overhead (fire-and-forget, Redis deduplication)
- Allow admins to block specific user-agents per group via admin API and UI
- Enforce UA blocks in the proxy with a 403 response before forwarding to upstream
- Provide autocomplete in the admin UI populated from recorded UAs

**Non-Goals:**
- Wildcard or regex UA matching — exact string match only
- Global (cross-group) UA blocking — per-group only
- Rate limiting by UA — blocking is binary (allowed or denied)
- Retroactive blocking of already-completed requests

## Decisions

**Fire-and-forget UA recording via tokio::spawn**
UA recording must not add latency to the proxy path. Using `tokio::spawn` with a cloned Redis pool and DB pool ensures the recording happens asynchronously. Failures are logged but do not affect the request. Alternative: a background channel (like uptime_tx) — rejected because it adds complexity for a non-critical side effect.

**Redis SADD for deduplication before DB insert**
`SADD group:{group_id}:user_agents {ua}` returns 1 only for new members, so the DB insert is skipped for already-seen UAs. This avoids a DB write on every request for common UAs. The DB insert uses `ON CONFLICT DO NOTHING` as a safety net for races (e.g., Redis eviction). Alternative: always insert with ON CONFLICT — rejected because it generates unnecessary DB load.

**blocked_user_agents rides in serialized GroupConfig**
The blocked list is loaded once during `resolve_group_config` and cached in Redis as part of the serialized GroupConfig (same as allowed_models). This means the proxy check is a pure in-memory Vec lookup with no Redis or DB access per request. Cache invalidation via `invalidate_group_all_keys` ensures changes take effect within one cache TTL cycle.

**DELETE /blocked uses request body, not path param**
User-Agent strings contain characters that are problematic in URL path segments (slashes, parentheses, etc.). Using a JSON body for DELETE avoids encoding issues. This is consistent with the feature requirements and acceptable for an internal admin API.

**Migration 038 — two separate tables**
`group_user_agents` (recording) and `group_blocked_user_agents` (blocking) are separate tables because they have different lifecycles and access patterns. Recorded UAs are append-only with `first_seen_at`; blocked UAs are managed by admins with `created_at`. Combining them with a `blocked` boolean would complicate queries and make the intent less clear.

**New admin route file group_user_agents.rs**
Follows the exact pattern of `group_allowed_models.rs` — a dedicated file for the nested resource, registered in `admin/mod.rs` and nested in `admin/groups.rs`. This keeps each resource's handlers self-contained.

## Risks / Trade-offs

**Redis Set growth** → The `group:{group_id}:user_agents` set grows unboundedly as new UAs are seen. For groups with many distinct clients this could consume significant Redis memory. Mitigation: the set is only used for deduplication (SADD return value check); the authoritative data is in the DB. If needed, a future cleanup job can trim the set. No mitigation needed for MVP.

**UA spoofing** → Clients can trivially set any User-Agent string. This feature is not a security boundary — it is an operational tool for admins to block known misbehaving clients. Mitigation: documented as a best-effort control, not a security guarantee.

**Cache staleness window** → After an admin blocks a UA, requests using that UA continue until the GroupConfig cache entry expires or is invalidated. `invalidate_group_all_keys` is called on every block/unblock mutation, so staleness is bounded by the time between the invalidation and the next cache population. This is the same trade-off as allowed_models and is acceptable.

**Empty/missing UA normalization** → Normalizing missing UA to "(empty)" means all requests without a UA header share one entry. If an admin blocks "(empty)", all clients without a UA header are blocked. This is intentional and documented.

## Migration Plan

1. Deploy migration 038 (additive — new tables only, no changes to existing tables)
2. Deploy backend with new cache function, GroupConfig field, proxy changes, and admin routes
3. Deploy frontend with new store functions and UI tab
4. No rollback complexity — new tables can be dropped, new GroupConfig field has `#[serde(default)]` so old cached entries deserialize cleanly

## Open Questions

None — all decisions are specified in the feature requirements.
