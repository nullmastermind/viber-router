## [2026-03-25] Round 1 (from spx-apply auto-verify)

### spx-uiux-verifier
- Fixed: [CRITICAL] Added `aria-label` to server toggle switch in GroupDetailPage.vue for screen-reader accessibility
- Fixed: [CRITICAL] Added try/catch with rollback and error notification to `toggleServerEnabled` in GroupDetailPage.vue — prevents UI from showing wrong visual state on API failure

### spx-arch-verifier
- Fixed: [WARNING] Updated `bulk_assign_server` upsert in routes/admin/groups.rs to include `is_enabled = true` in both INSERT column list and ON CONFLICT DO UPDATE clause — prevents silently leaving re-assigned servers in disabled state

## [2026-03-25] Round 2 (from spx-apply re-verify)

### spx-verifier
- Fixed: [CRITICAL] Corrected rollback logic in `toggleServerEnabled` — changed from capturing `previous = s.is_enabled` (which was already the post-toggle value) to `s.is_enabled = !s.is_enabled` in catch block for correct UI revert on API failure
