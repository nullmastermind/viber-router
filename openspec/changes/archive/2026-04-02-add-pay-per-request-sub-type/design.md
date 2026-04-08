## Context

The viber-router subscription system currently supports two billing types: `fixed` (lifetime budget) and `hourly_reset` (budget resets every N hours). Both types calculate cost from token usage via a formula involving per-model pricing from the `models` table.

Admins need a third billing model where each API call has a flat cost determined by the model, independent of token count. This is useful for simplified pricing tiers where the admin wants predictable per-call revenue.

The existing windowing logic in `subscription.rs` is tied to `sub_type == "hourly_reset"` string checks. Since `pay_per_request` also needs optional windowing, this check must be generalized.

## Goals / Non-Goals

**Goals:**
- Add `pay_per_request` as a valid `sub_type` in both `subscription_plans` and `key_subscriptions`
- Add `model_request_costs` JSONB field to both tables to store flat per-model costs
- Block requests to models not listed in `model_request_costs` when subscription is `pay_per_request`
- Apply flat cost (not token-based) in post-response billing for `pay_per_request`
- Support optional `reset_hours` for `pay_per_request` (reuses existing windowing logic)
- Generalize windowing condition from `sub_type == "hourly_reset"` to `reset_hours.is_some()`
- Snapshot `model_request_costs` when assigning a plan to a sub-key
- Frontend UI for configuring and displaying `model_request_costs`

**Non-Goals:**
- Pre-request cost reservation or locking (existing post-request billing pattern is retained)
- Per-request cost that varies by token count for `pay_per_request` type
- Changing the billing flow for `fixed` or `hourly_reset` types

## Decisions

### D1: Separate `model_request_costs` from `model_limits`

`model_limits` is a per-model budget cap (how much total can be spent on a model). `model_request_costs` is a per-model flat cost per call. These are orthogonal concerns. Merging them would require a more complex schema and break the existing `model_limits` semantics. They are kept as separate JSONB columns.

Alternatives considered: a single `model_config` JSONB with nested objects per model — rejected because it complicates existing `model_limits` reads and adds unnecessary nesting.

### D2: Block request if model not in `model_request_costs`

For `pay_per_request`, every model must have an explicit price. Allowing unlisted models would mean free requests, which defeats the billing purpose. The subscription is skipped (not the request blocked globally) — if another subscription covers the request, it proceeds.

Alternatives considered: default cost of 0 for unlisted models — rejected because it silently allows free usage.

### D3: Generalize windowing to `reset_hours.is_some()`

The current code checks `sub_type == "hourly_reset"` in multiple places to decide whether to use windowed Redis keys. Since `pay_per_request` can also have `reset_hours`, the check is generalized to `reset_hours.is_some()`. This is backward compatible: `hourly_reset` always has `reset_hours` set (enforced by validation), so existing behavior is unchanged.

Alternatives considered: adding `pay_per_request` as another string check — rejected because it would require updating every windowing check site and is fragile.

### D4: Post-request flat cost replaces token-based calculation

In `proxy.rs`, after the response returns, if the active subscription is `pay_per_request`, the cost is `model_request_costs[model]` instead of the token-based formula. The existing `calculate_cost()` function is not called for this type. This keeps the billing path clean and avoids needing model pricing data for `pay_per_request` subscriptions.

### D5: `pay_per_request` priority in subscription selection

`pay_per_request` is placed after `hourly_reset` and before `fixed` in the selection priority. Rationale: it behaves like a windowed subscription when `reset_hours` is set, and like a fixed subscription when not. Placing it between the two is a reasonable default. Admins can control priority by having only one type active at a time.

## Risks / Trade-offs

- [Budget overrun by 1 request] The existing post-request billing pattern means a `pay_per_request` subscription can be charged one request beyond its budget. This is the same risk as existing types and is accepted. Mitigation: none (consistent with existing behavior).
- [Model name mismatch] If the model name in the request doesn't exactly match the key in `model_request_costs`, the request is blocked. Mitigation: frontend model selector uses the same model list as the proxy, ensuring consistent naming.
- [Windowing generalization] Changing `sub_type == "hourly_reset"` to `reset_hours.is_some()` affects all three cost functions and the counter update. If a `fixed` plan ever accidentally gets `reset_hours` set, it would use windowed keys. Mitigation: validation ensures `fixed` plans cannot have `reset_hours` set (existing constraint).

## Migration Plan

1. Deploy migration adding `model_request_costs` column with `DEFAULT '{}'` — safe, no downtime, existing rows get empty object
2. Deploy migration updating CHECK constraints to include `pay_per_request` — safe, additive
3. Deploy backend with new type support
4. Deploy frontend with new UI

Rollback: revert backend deploy; the column and constraint change are backward compatible with the old code (old code ignores the new column and never writes `pay_per_request`).

## Open Questions

None — all decisions are made per the feature brief.
