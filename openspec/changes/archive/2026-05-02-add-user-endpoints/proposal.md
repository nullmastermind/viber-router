## Why

End users who hold sub-keys currently cannot self-service route their traffic through their own backup or preferred API servers. Adding User Endpoints gives sub-key holders a controlled way to add custom endpoints from the public usage page without conflating them with admin-provisioned bonus subscriptions.

## What Changes

- Add a new User Endpoints capability for sub-key holders to create, list, update, toggle, and delete custom API endpoints from `/usage`.
- Store user-managed endpoints in a new `user_endpoints` database table and associate usage logs with `user_endpoint_id`.
- Add public API routes authenticated by the same sub-key query parameter used by public usage APIs.
- Integrate enabled, model-compatible user endpoints into proxy routing as priority endpoints before bonus servers and fallback endpoints after blocked subscriptions or exhausted group servers.
- Surface user endpoint quota and recent usage data in the public usage response and page UI.
- Enforce a maximum of 10 user endpoints per sub-key.
- Preserve existing bonus subscription behavior and keep admin visibility of user endpoints out of scope.

## Capabilities

### New Capabilities
- `user-endpoints`: End-user management and proxy routing for custom API endpoints tied to a sub-key.

### Modified Capabilities
- `public-usage-api`: Include user endpoint data, quota information, and usage statistics in public usage responses.
- `public-usage-page`: Add public UI controls for managing custom endpoints on the usage page.
- `proxy-engine`: Route requests through priority and fallback user endpoints while preserving original request paths and model mapping behavior.
- `token-usage-storage`: Attribute routed usage to `user_endpoint_id` when traffic is served by a user endpoint.

## Impact

- Backend database migration `046` adds `user_endpoints`, `token_usage_logs.user_endpoint_id`, and supporting indexes.
- Backend models, cache helpers, public routes, usage aggregation, proxy routing, and usage buffering change.
- Frontend `PublicUsagePage.vue` gains a Custom Endpoints management section and dialogs.
- Required checks include `just check` and Rust unit tests for user endpoint model filtering.
