## Context

Bonus subscriptions already support optional model allowlists at creation time and during request eligibility checks. The current administrative workflow stops at creation: once a bonus subscription exists, the admin UI can display `bonus_allowed_models`, but there is no UI or backend mutation path that changes the value. The existing admin PATCH route is cancellation-focused and should not be broadened into a generic edit surface for unrelated subscription fields.

## Goals / Non-Goals

**Goals:**

- Add a focused backend update path for `bonus_allowed_models` on active bonus subscriptions.
- Keep validation narrow: the subscription must exist, belong to the requested key, be `bonus` type, and be active.
- Invalidate the existing Redis subscription cache for the affected key after a successful update.
- Add an inline frontend edit affordance for the allowed models cell using the group's allowed model list as options.
- Refresh the subscription list after successful save so table state matches the server.

**Non-Goals:**

- Do not change bonus subscription creation behavior.
- Do not add editing for bonus name, base URL, API key, token limits, or dates.
- Do not change subscription enforcement semantics beyond updating the stored allowlist.
- Do not introduce new persistence tables or migrations.

## Decisions

1. Use a dedicated `PUT /{sub_id}/bonus-allowed-models` endpoint.
   - Rationale: the existing PATCH route is cancellation-oriented. A dedicated endpoint keeps the mutation constrained to one field and avoids implying support for general subscription edits.
   - Alternative considered: extend the existing PATCH endpoint. This would reuse a route but make validation and request semantics less explicit.

2. Represent the update payload with `UpdateBonusSubscription { bonus_allowed_models: Option<Vec<String>> }`.
   - Rationale: this mirrors the existing storage field shape and supports both omitted/null and array payloads. The route will accept the requested `{ bonus_allowed_models: string[] }` shape while keeping the model flexible enough to normalize empty arrays to unrestricted behavior.
   - Alternative considered: create a non-optional `Vec<String>` request type. This would enforce the request shape more strictly but would not align with the existing nullable database semantics.

3. Restrict updates to active bonus subscriptions belonging to the selected key.
   - Rationale: only active bonus subscriptions participate in routing, and editing canceled or regular/manual subscriptions would create confusing state or unsupported behavior.
   - Alternative considered: allow edits to inactive bonus subscriptions. This adds no immediate user value and complicates rules around historical records.

4. Use the existing group allowed model options for the frontend editor.
   - Rationale: creation already uses this source of truth, and it prevents admins from selecting models outside the group's configured model set.
   - Alternative considered: free-text editing. This would allow arbitrary values but would be inconsistent with the Add Bonus dialog and increase typo risk.

5. Treat an empty selected list as unrestricted/all models.
   - Rationale: existing semantics define both `NULL` and `[]` as accepting all models. The UI should preserve that mental model by allowing admins to clear all selections to mean all models.

## Risks / Trade-offs

- Stale Redis cache after updates → mitigate by deleting `key_subs:{key_id}` using the established invalidation pattern immediately after the database update succeeds.
- Frontend editor may present stale group model options if allowed models change concurrently → mitigate by using currently loaded group data and refreshing subscriptions after save; broader real-time sync is out of scope.
- Empty array versus null normalization may be inconsistent across layers → mitigate by documenting and preserving both as unrestricted/all models in API behavior and UI display.
- Dedicated endpoint adds one more API route → accepted because it gives clearer authorization and validation boundaries than a broad PATCH mutation.
