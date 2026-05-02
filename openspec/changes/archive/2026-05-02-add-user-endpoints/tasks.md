## 1. Database and Data Model

- [x] 1.1 Add migration `046` to create `user_endpoints` with the required columns, defaults, constraints, `group_key_id` cascade reference, and ordering/index support
- [x] 1.2 Extend `token_usage_logs` with nullable `user_endpoint_id` and add an index on `user_endpoint_id`
- [x] 1.3 Implement `viber-router-api/src/models/user_endpoint.rs` with request/response structs, database row types, validation helpers, and model compatibility helpers
- [x] 1.4 Register the user endpoint model module in `viber-router-api/src/models/mod.rs` and ensure SQLx types compile ← (verify: migration schema, Rust structs, and SQLx queries agree on field names and JSON/nullability)

## 2. Backend Public API and Cache

- [x] 2.1 Add cache helpers in `viber-router-api/src/cache.rs` for `user_eps:{group_key_id}` with a 300-second TTL and mutation invalidation
- [x] 2.2 Implement `viber-router-api/src/routes/public/user_endpoints.rs` with list/create/patch/delete handlers authenticated by `key=<sub-key>`
- [x] 2.3 Enforce ownership, active sub-key validation, JSON object validation, priority mode validation, required create fields, and max 10 endpoints per sub-key
- [x] 2.4 Register user endpoint routes in `viber-router-api/src/routes/public/mod.rs`
- [x] 2.5 Add quota fetching and 30-day per-model usage aggregation for user endpoint list responses ← (verify: CRUD routes cannot access another sub-key's endpoints and cache invalidates after every mutation)

## 3. Public Usage API Integration

- [x] 3.1 Extend public usage response types in `viber-router-api/src/routes/public/usage.rs` to include `user_endpoints`
- [x] 3.2 Populate each user endpoint with display configuration, optional quota data, enabled state, priority mode, and 30-day per-model usage stats
- [x] 3.3 Preserve existing subscription, bonus quota, bonus usage, rate limit, and invalid-key behavior ← (verify: existing `/api/public/usage` fields remain backward compatible and `user_endpoints` is `[]` when none exist)

## 4. Proxy Routing Integration

- [x] 4.1 Load enabled user endpoints for the resolved sub-key/group key using the new cache helpers
- [x] 4.2 Add user endpoint model filtering where empty `model_mappings` accepts all models and non-empty mappings require the request model as a key
- [x] 4.3 Implement user endpoint forwarding modeled after group servers: preserve path/query, transform request body, forward safe headers, set `x-api-key` and `authorization`, and detect Anthropic/OpenAI endpoint type
- [x] 4.4 Attempt priority user endpoints in FIFO `created_at ASC` order before bonus servers and return immediately on 2xx
- [x] 4.5 Attempt fallback user endpoints in FIFO `created_at ASC` order when subscription enforcement blocks or all group servers fail
- [x] 4.6 Ensure fallback exhaustion returns the existing path-appropriate 429 response and does not alter bonus or normal group-server waterfall semantics ← (verify: proxy flow order matches priority endpoints, bonus, subscription check, group servers, fallback endpoints, final 429)

## 5. Usage Logging Integration

- [x] 5.1 Add `user_endpoint_id` to `TokenUsageEntry` in `viber-router-api/src/usage_buffer.rs` and persist it to `token_usage_logs`
- [x] 5.2 Enqueue usage for user endpoint non-streaming responses with `user_endpoint_id` set and `subscription_id = NULL`
- [x] 5.3 Wrap user endpoint streaming responses with existing usage tracking, using `Uuid::nil()` where a server identifier is required and persisting the real `user_endpoint_id`
- [x] 5.4 Keep group server and bonus server usage logs with `user_endpoint_id = NULL` ← (verify: token usage rows correctly distinguish subscription usage from user endpoint usage)

## 6. Frontend Public Usage Page

- [x] 6.1 Update public usage page types and API calls in `src/pages/PublicUsagePage.vue` to consume and refresh `user_endpoints`
- [x] 6.2 Add the Custom Endpoints section between subscription cards and the usage table, with cards showing name, truncated base URL, priority badge, enabled toggle, edit/delete actions, quota data, and usage stats
- [x] 6.3 Add persistent add/edit dialogs following existing Quasar dialog patterns with Name, Base URL, API Key, Model Mappings JSON, Priority Mode, Quota URL, and Quota Headers JSON fields
- [x] 6.4 Implement client-side JSON validation for Model Mappings and Quota Headers plus required-field validation for create
- [x] 6.5 Implement create, update, toggle, and delete flows with loading state and success/error notifications
- [x] 6.6 Enforce and display the max 10 endpoints limit in the UI and handle server max-limit responses clearly ← (verify: public `/usage` UX can add, edit, toggle, delete, and refresh endpoints without admin authentication)

## 7. Tests and Checks

- [x] 7.1 Add Rust unit tests for user endpoint model compatibility and mapping behavior
- [x] 7.2 Add backend tests or targeted coverage for CRUD ownership/max-limit behavior where practical
- [x] 7.3 Run `just check` and fix all cargo, clippy, TypeScript, and Biome issues
- [x] 7.4 Manually verify representative proxy flows for priority success, priority failure to bonus/group servers, blocked subscription fallback, and group-server exhaustion fallback ← (verify: all required checks pass and routing behavior matches the specs)
