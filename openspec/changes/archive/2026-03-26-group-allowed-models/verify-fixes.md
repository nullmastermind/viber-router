## [2026-03-26] Round 1 (from spx-apply auto-verify)

### spx-verifier
- Fixed: Initialized `allowedModels` from `group.value.allowed_models` in `loadGroup()` so key model section is visible without visiting the Allowed Models tab first (GroupDetailPage.vue)

### spx-arch-verifier
- Fixed: Replaced SELECT+INSERT find-or-create with `INSERT ... ON CONFLICT DO UPDATE` upsert in `add_group_allowed_model` to handle concurrent requests safely (group_allowed_models.rs)
- Fixed: Changed `.unwrap_or_default()` to `.ok()?` for allowed_models and key_allowed_models queries in `resolve_group_config` to propagate DB errors instead of silently degrading security (proxy.rs)

### spx-uiux-verifier
- Fixed: Added `aria-label` to remove button in allowed models list (`Remove ${m.name}`) (GroupDetailPage.vue)
- Fixed: Added `aria-label="Create and add model"` to inline add button in model picker (GroupDetailPage.vue)
