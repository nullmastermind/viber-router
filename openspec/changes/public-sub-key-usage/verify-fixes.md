## [2026-03-27] Round 1 (from spx-apply auto-verify)

### spx-verifier
- Fixed: `cost_used` for non-active subscriptions (expired/cancelled/exhausted) now returns `0.0` instead of actual accumulated cost, matching spec requirement in `public-usage-api/spec.md`
- Fixed: Rate limit increment moved after key parameter validation so empty-key requests don't consume rate limit quota

### spx-uiux-verifier
- Fixed: Error message on key input form now uses manual `<div>` below input (LoginPage pattern) instead of Quasar's `:error-message` which is hidden by global `.q-field__bottom { display: none }` rule
- Fixed: Inactive subscription card opacity increased from 0.5 to 0.6 to maintain WCAG AA contrast compliance
- Fixed: Added `aria-label`, `aria-valuenow`, `aria-valuemin`, `aria-valuemax` to `q-linear-progress` for screen reader accessibility
- Fixed: Added `aria-label` to key input field
