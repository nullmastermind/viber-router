## ADDED Requirements

### Requirement: Bonus subscription sort order column
The `key_subscriptions` table SHALL include a `sort_order` column of type `INTEGER NOT NULL DEFAULT 0`. Migration `049_add_sort_order_to_key_subscriptions.sql` SHALL add this column without modifying existing data. The `KeySubscription` Rust model SHALL expose `sort_order: i32`.

#### Scenario: Existing rows default to zero
- **WHEN** migration 049 is applied to a database that already contains key_subscriptions rows
- **THEN** every existing row SHALL have `sort_order = 0` and no row SHALL be modified apart from the default assignment

#### Scenario: New bonus subscription gets default sort order
- **WHEN** a bonus subscription is inserted via the existing `assign_subscription` path
- **THEN** the resulting row SHALL have `sort_order = 0` (no insert-time ordering logic is introduced)

### Requirement: Reorder bonus subscriptions endpoint
The system SHALL expose `PUT /api/admin/groups/:group_id/keys/:key_id/subscriptions/reorder` that accepts a JSON body `{ "ordered_ids": [uuid, ...] }`. The endpoint SHALL validate that `ordered_ids` contains exactly the set of `key_subscriptions.id` values belonging to `:key_id` with `sub_type = 'bonus'` (cancelled bonuses included). On valid input, the endpoint SHALL update each listed row's `sort_order` to its zero-based index in the array within a single database transaction. After the transaction commits, the endpoint SHALL delete the Redis cache key `key_subs:{key_id}` and return HTTP 200 with an empty body or `{ "ok": true }`.

#### Scenario: Valid reorder with all bonus ids
- **WHEN** an admin sends `{ "ordered_ids": [b_id_2, b_id_1, b_id_3] }` and the key has exactly bonus subs `b_id_1`, `b_id_2`, `b_id_3`
- **THEN** the endpoint SHALL set `sort_order` to `0`, `1`, `2` for `b_id_2`, `b_id_1`, `b_id_3` respectively, invalidate `key_subs:{key_id}`, and return HTTP 200

#### Scenario: Reorder with cancelled bonus included
- **WHEN** a key has active bonus `b_a` and cancelled bonus `b_b`, and the admin sends `{ "ordered_ids": [b_b, b_a] }`
- **THEN** the endpoint SHALL update both rows (sort_order 0 for `b_b`, 1 for `b_a`) and return HTTP 200

#### Scenario: Ordered ids missing a bonus id
- **WHEN** the key has bonus subs `b_1`, `b_2` and the admin sends `{ "ordered_ids": [b_1] }`
- **THEN** the endpoint SHALL return HTTP 400 with an error indicating the list must contain every bonus subscription for the key

#### Scenario: Ordered ids contains an id not belonging to the key
- **WHEN** the admin sends an `ordered_ids` array containing a UUID that does not exist in `key_subscriptions` with `group_key_id = :key_id`
- **THEN** the endpoint SHALL return HTTP 400 with an error indicating an unknown id

#### Scenario: Ordered ids contains a non-bonus subscription id
- **WHEN** the admin sends an `ordered_ids` array containing a `key_subscriptions.id` whose `sub_type != 'bonus'`
- **THEN** the endpoint SHALL return HTTP 400 with an error indicating only bonus subscriptions can be reordered

#### Scenario: Ordered ids contains duplicates
- **WHEN** the admin sends `{ "ordered_ids": [b_1, b_1, b_2] }`
- **THEN** the endpoint SHALL return HTTP 400 with an error indicating duplicate ids

### Requirement: Admin subscription list ordering
The admin list endpoint `GET /api/admin/groups/:group_id/keys/:key_id/subscriptions` SHALL order rows by `(sub_type = 'bonus') DESC, sort_order ASC, created_at DESC`. This places bonus subscriptions at the top of the table in admin-controlled order while leaving non-bonus subscriptions in their existing most-recent-first order.

#### Scenario: Bonus subscriptions appear before non-bonus
- **WHEN** a key has two bonus subs and one fixed sub
- **THEN** the admin list response SHALL list the two bonus subs (in `sort_order ASC` order) before the fixed sub

#### Scenario: Bonus subscriptions with equal sort_order
- **WHEN** a key has two bonus subs both with `sort_order = 0`
- **THEN** the admin list response SHALL order them by `created_at DESC` as the final tiebreaker

### Requirement: Admin UI reorder controls on bonus rows
The subscriptions table in `GroupDetailPage.vue` SHALL render up and down arrow buttons on every row whose `sub_type === 'bonus'`. The up button SHALL be disabled for the first bonus row and the down button SHALL be disabled for the last bonus row. Clicking either button SHALL compute the new ordered list of bonus subscription ids for the current sub-key by swapping the clicked row with its neighbor, then issue a `PUT /subscriptions/reorder` request. While the request is in flight the buttons SHALL show a loading state. On HTTP 2xx the component SHALL refetch the subscription list so the table reflects persisted order. On error the component SHALL show a negative toast and refetch to revert the optimistic state.

#### Scenario: Click up on a middle bonus row
- **WHEN** the admin clicks the up button on the second bonus row
- **THEN** the UI SHALL call `PUT /subscriptions/reorder` with the first and second bonus ids swapped and refetch the list on success

#### Scenario: Up disabled on first bonus row
- **WHEN** the first bonus row is rendered
- **THEN** its up button SHALL be disabled

#### Scenario: Down disabled on last bonus row
- **WHEN** the last bonus row is rendered
- **THEN** its down button SHALL be disabled

#### Scenario: Reorder API fails
- **WHEN** `PUT /subscriptions/reorder` returns a non-2xx response
- **THEN** the UI SHALL display a negative toast and refetch the subscription list so the displayed order matches server state

### Requirement: Public usage bonus rendering order
The public usage API (`GET /api/usage?key=...`) SHALL return subscriptions ordered by `status = 'active' DESC, (sub_type = 'bonus') DESC, sort_order ASC, created_at DESC`. Active bonus subscriptions SHALL therefore render before non-bonus subscriptions, in admin-defined order. The `PublicUsagePage.vue` component SHALL render bonus cards in the order returned by the API with no additional sorting.

#### Scenario: Bonus cards follow sort_order
- **WHEN** a sub-key has bonus subs with `sort_order = 0, 1, 2` returned in that order
- **THEN** the public usage page SHALL render their cards in the same 0, 1, 2 order

## MODIFIED Requirements

### Requirement: Bonus server list in SubCheckResult
The subscription engine's `check_subscriptions()` function SHALL separate bonus subscriptions from non-bonus subscriptions. If any active bonus subscriptions are eligible for the request model, the function SHALL return `BonusServers { servers, fallback_subscription }`. `servers` SHALL be the list of active bonus subs that accept the request model ordered by `sort_order ASC, created_at ASC` (admin-defined order, tiebreak by creation time), each as a `BonusServer { subscription_id, base_url, api_key, name, allowed_models }`. `allowed_models` SHALL contain the stored bonus model allowlist or an empty list for unrestricted bonus subscriptions. `fallback_subscription` SHALL be `Some((sub_id, rpm_limit))` if the non-bonus check would have returned `Allowed`, or `None` otherwise.

#### Scenario: Sub-key has one bonus sub and one fixed sub with budget
- **WHEN** `check_subscriptions()` is called for a sub-key with one active bonus sub that accepts the request model and one active fixed sub with remaining budget
- **THEN** the function SHALL return `BonusServers { servers: [bonus], fallback_subscription: Some((fixed_sub_id, None)) }`

#### Scenario: Sub-key has only bonus subs
- **WHEN** `check_subscriptions()` is called for a sub-key with only active bonus subscriptions that accept the request model
- **THEN** the function SHALL return `BonusServers { servers: [bonus1, bonus2, ...], fallback_subscription: None }` where `servers` is ordered by `sort_order ASC` then `created_at ASC`

#### Scenario: Sub-key has bonus subs and exhausted non-bonus subs
- **WHEN** all non-bonus subscriptions are exhausted or expired and at least one bonus sub accepts the request model
- **THEN** the function SHALL return `BonusServers { servers: [bonus], fallback_subscription: None }`

#### Scenario: Sub-key has no eligible bonus subs
- **WHEN** `check_subscriptions()` is called for a sub-key with no active bonus subscriptions that accept the request model
- **THEN** the function SHALL NOT return `BonusServers`; existing Allowed/Blocked logic is unchanged

#### Scenario: Sub-key has no bonus subs
- **WHEN** `check_subscriptions()` is called for a sub-key with no bonus subscriptions
- **THEN** the function SHALL NOT return `BonusServers`; existing Allowed/Blocked logic is unchanged

#### Scenario: Bonus subs ordered by admin sort_order
- **WHEN** a key has bonus subs `b_x` (`sort_order = 2`) and `b_y` (`sort_order = 0`) both eligible for the request model
- **THEN** `servers` SHALL be `[b_y, b_x]`

### Requirement: Bonus subscription routing in proxy
When a sub-key's subscription check returns `BonusServers`, the proxy SHALL try each bonus server in the order given (admin-defined `sort_order ASC`, tiebreak `created_at ASC`) before the group server waterfall. For each bonus server, the proxy SHALL build the upstream URL as `bonus_base_url.trim_end_matches('/') + "/v1/messages"`, set headers `x-api-key: bonus_api_key` and `authorization: Bearer bonus_api_key`, and forward the transformed request body. A 2xx response from a bonus server SHALL be returned immediately. A non-2xx response SHALL be logged and the proxy SHALL continue to the next bonus server.

#### Scenario: Bonus server returns 2xx
- **WHEN** the first bonus server (per sort order) returns HTTP 200
- **THEN** the proxy SHALL return the response to the client immediately without trying group servers

#### Scenario: Bonus server returns non-2xx, fallback to next bonus
- **WHEN** the first bonus server returns HTTP 529 and a second bonus server exists
- **THEN** the proxy SHALL log the failure and attempt the second bonus server (next in sort order)

#### Scenario: All bonus servers exhausted, fallback subscription exists
- **WHEN** all bonus servers return non-2xx and `fallback_subscription` is Some(sub_id, rpm_limit)
- **THEN** the proxy SHALL set `selected_subscription_id = sub_id` and proceed to the group server waterfall as if the sub-key had a normal subscription

#### Scenario: All bonus servers exhausted, no fallback subscription
- **WHEN** all bonus servers return non-2xx and `fallback_subscription` is None
- **THEN** the proxy SHALL proceed to the group server waterfall without subscription tracking (unlimited access to group servers)

#### Scenario: Bonus usage logging — cost is NULL
- **WHEN** a bonus server returns HTTP 200 and token usage is recorded
- **THEN** `token_usage_logs.cost_usd` SHALL be NULL and `token_usage_logs.subscription_id` SHALL be set to the bonus subscription's UUID
