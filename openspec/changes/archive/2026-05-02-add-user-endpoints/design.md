## Context

Viber Router already supports admin-provisioned bonus subscriptions and normal group-server proxy waterfalls. Sub-key holders can view their public usage data from `/usage`, but they cannot self-manage custom upstream endpoints. User Endpoints introduce a separate user-owned routing concept, tied to `group_keys`, surfaced on the public usage page, and used by the proxy before bonus servers for priority mode or after blocked/exhausted routing paths for fallback mode.

This change spans database schema, public APIs, proxy routing, usage logging, caching, and the public Vue page. It must preserve existing bonus behavior and group-server behavior while adding an isolated `user_endpoints` table and usage attribution via `token_usage_logs.user_endpoint_id`.

## Goals / Non-Goals

**Goals:**

- Allow sub-key holders to create, list, update, enable/disable, and delete up to 10 custom API endpoints from `/usage`.
- Store user endpoints separately from bonus subscriptions and associate each endpoint with the owning `group_key_id`.
- Route enabled, model-compatible priority endpoints before bonus servers and fallback endpoints after subscription blockage or group-server exhaustion.
- Preserve the original proxied request path and query string when forwarding to user endpoints.
- Reuse existing request body model transformation and SSE usage tracking patterns where possible.
- Include quota data and 30-day usage statistics for user endpoints in public usage responses and UI.
- Attribute token usage served by user endpoints with `user_endpoint_id` and no subscription charge.

**Non-Goals:**

- Admin visibility or management for user endpoints.
- Extending `key_subscriptions` or bonus subscription semantics.
- New allowed-model fields beyond `model_mappings` keys.
- Health checks, retry configuration, or circuit breakers for user endpoints.
- SSE parser support beyond existing Anthropic/OpenAI detection.

## Decisions

### Store user endpoints in a dedicated table

User endpoints will use a new `user_endpoints` table keyed by `group_key_id`, with endpoint credentials, model mappings, optional quota configuration, priority mode, enabled state, and timestamps. This keeps end-user-managed infrastructure separate from admin-managed bonus subscriptions.

Alternatives considered:
- Extending `key_subscriptions`: rejected because bonus subscriptions are admin-provisioned and carry subscription semantics that do not apply to user-owned endpoints.
- Storing JSON on `group_keys`: rejected because endpoint lifecycle, querying, usage joins, and limits are cleaner with relational rows.

### Authenticate public endpoint management by sub-key query parameter

The user endpoint API will follow the existing public usage pattern by requiring `?key=<sub-key>` and resolving it to an active `group_key`. CRUD operations must only affect endpoints owned by that resolved `group_key_id`.

Alternatives considered:
- New user auth/session layer: rejected as out of scope and inconsistent with current public usage access.
- API key in headers: rejected to match existing `/api/public/usage` behavior.

### Enforce model compatibility through mapping keys

A user endpoint with non-empty `model_mappings` only accepts requests whose source model is present as a key. Empty mappings accept all models. When a mapping exists, the existing `transform_request_body()` behavior should map the source model to the endpoint target model.

Alternatives considered:
- Separate allowed models list: rejected because model mapping keys already provide the filter.
- Always route to all endpoints: rejected because endpoints often support only specific model names.

### Treat priority and fallback endpoints as two FIFO waterfalls

Enabled, compatible priority endpoints are attempted in `created_at ASC` order before existing bonus servers. Enabled, compatible fallback endpoints are attempted in `created_at ASC` order after subscription blockage or after all group servers fail. A 2xx response returns immediately; non-2xx or connection failures continue through the endpoint waterfall.

Alternatives considered:
- Try fallback endpoints before subscription checks: rejected because fallback endpoints are intended as last safety net after quota exhaustion or upstream failure.
- Merge user endpoints with group servers: rejected because user endpoints have different ownership, logging, and no admin server configuration.

### Model user endpoint forwarding after group server proxying

User endpoint requests should preserve the original path and query string, forward safe headers while excluding auth/host/content-length, set both `x-api-key` and `authorization`, apply model transformation, detect Anthropic vs OpenAI parsing via existing endpoint detection, and use the same usage extraction/wrapping patterns as group servers.

Alternatives considered:
- Reuse bonus proxy logic: rejected because bonus currently hard-codes a request path and does not match the required path preservation behavior.
- Create a totally separate proxy stack: rejected because it would duplicate usage tracking and streaming behavior.

### Cache user endpoint configuration by group key

Cache helpers should store endpoint configuration under `user_eps:{group_key_id}` with a 300-second TTL. Mutation routes must invalidate or refresh this cache to avoid stale routing decisions after users edit, toggle, or delete endpoints.

Alternatives considered:
- No cache: acceptable for correctness but inconsistent with existing proxy performance patterns.
- Cache by sub-key string: rejected because `group_key_id` is stable and avoids storing sensitive key material in cache keys.

## Risks / Trade-offs

- [Risk] Public CRUD endpoints expose user-owned API keys in responses if not carefully shaped. → Mitigation: return enough data for editing by the trusted sub-key holder, but never expose endpoints across group keys; keep admin visibility out of scope.
- [Risk] User endpoint waterfalls could add latency before or after normal routing. → Mitigation: try only enabled and model-compatible endpoints, preserve FIFO ordering, and stop on first 2xx response.
- [Risk] Cache staleness could route to deleted or disabled endpoints for up to 300 seconds. → Mitigation: invalidate cache on create/update/delete/toggle operations.
- [Risk] Usage attribution can be ambiguous if `server_id` remains mandatory in existing tracking helpers. → Mitigation: use `Uuid::nil()` for `server_id` in user endpoint streaming wrappers and persist `user_endpoint_id` separately, with `subscription_id = NULL`.
- [Risk] Schema migration touches a partitioned log table. → Mitigation: add nullable `user_endpoint_id` and index in migration 046 without backfilling existing data.

## Migration Plan

1. Add migration `046` creating `user_endpoints`, adding nullable `token_usage_logs.user_endpoint_id`, and adding required indexes.
2. Deploy backend code that can read/write the new table, expose public management routes, include user endpoint usage in public usage responses, and route via user endpoint waterfalls.
3. Deploy frontend UI for Custom Endpoints on `/usage`.
4. Rollback by disabling the UI and route registration first; the nullable column and unused table can remain safely until a later cleanup migration.

## Open Questions

- Exact quota response normalization should match the existing bonus quota display shape when possible; implementation should document any unsupported upstream quota formats.
