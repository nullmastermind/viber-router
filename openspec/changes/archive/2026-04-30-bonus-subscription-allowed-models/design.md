## Context

Bonus subscriptions are stored in `key_subscriptions` and are selected by the subscription engine before the normal group server waterfall. Today, every active bonus subscription is considered eligible for every request model. Existing per-key and per-server model allowlists use model name strings, and the proxy request model is matched by name.

The admin group detail page already loads the group's allowed models as `Model[]` with `id` and `name`; the bonus subscription allowlist must store model names so backend routing can compare directly against request model names.

## Goals / Non-Goals

**Goals:**

- Add a nullable `bonus_allowed_models TEXT[]` column to `key_subscriptions`.
- Accept optional bonus allowed model names when an admin creates a bonus subscription.
- Enforce bonus allowed models during subscription checks before bonus servers are returned to proxy routing.
- Preserve backward compatibility: `NULL` and empty arrays mean the bonus subscription accepts all models.
- Expose and display the allowlist in the admin group detail UI.

**Non-Goals:**

- Changing normal subscription plan model limits or billing semantics.
- Validating bonus model names against an external provider's model catalog.
- Adding edit support for an existing bonus subscription allowlist unless already covered by existing subscription editing flows.
- Changing public usage attribution or public usage page footer content.

## Decisions

1. Store the allowlist as `bonus_allowed_models TEXT[]` on `key_subscriptions`.
   - Rationale: This matches the existing `key_allowed_models` pattern and avoids a join table for a small list of model names.
   - Alternative considered: Store model IDs. Rejected because proxy matching operates on request model names and existing allowlist behavior stores names.

2. Treat both `NULL` and empty arrays as unrestricted.
   - Rationale: Existing rows have no configured allowlist, and admins need a way to leave bonus subscriptions accepting all models.
   - Alternative considered: Empty array means deny all. Rejected because it would be surprising in a multi-select UI and could break existing behavior during migration.

3. Filter bonus subscriptions inside `check_subscriptions()` before constructing or returning `BonusServer` entries.
   - Rationale: Downstream proxy code should only see eligible bonus servers and can keep its FIFO retry/fallback behavior unchanged.
   - Alternative considered: Return all bonus servers with allowlists and filter later in proxy routing. Rejected because eligibility belongs in the subscription check and would duplicate model filtering concerns.

4. Use group's existing `allowedModels` list as the Add Bonus QSelect options and send model names.
   - Rationale: It constrains admin input to known group-allowed model names and matches backend request model comparisons.
   - Alternative considered: Free text model entry. Rejected because it increases typo risk and is inconsistent with the existing group model data.

## Risks / Trade-offs

- [Risk] Existing SELECT/INSERT projections for `key_subscriptions` can fail to compile if the new field is omitted from Rust structs or sqlx mappings. → Mitigation: update all affected projections and run `just check` during implementation.
- [Risk] Filtering after bonus server construction could let restricted bonus subscriptions be attempted for the wrong model. → Mitigation: implement filtering in the bonus partition/filter path before collecting eligible `BonusServer` values.
- [Risk] UI could send model IDs instead of names because the group model objects include both. → Mitigation: configure the multi-select to emit and submit `Model.name` strings.
- [Risk] Empty allowlists may be ambiguous. → Mitigation: document and test that `NULL` and `[]` both mean "All models".

## Migration Plan

- Add an idempotent migration using `ALTER TABLE key_subscriptions ADD COLUMN IF NOT EXISTS bonus_allowed_models TEXT[]`.
- Deploy backend code that reads, writes, and enforces the new nullable column.
- Deploy frontend code that optionally sends and displays the field.
- Rollback is safe at the application level by reverting code; the nullable column can remain unused. If required, a follow-up migration can drop the column after confirming no code references it.
