## [2026-03-26] Round 1 (from spx-apply auto-verify)

### spx-verifier
- Fixed: `is_rate_limited` in `rate_limiter.rs` — changed GET result type from `i64` to `Option<i64>` with `unwrap_or(0)` to correctly handle Redis nil (key absent) instead of treating it as an error

### spx-arch-verifier
- Fixed: Removed unused `_window_seconds` parameter from `is_rate_limited` function signature and updated call site in `proxy.rs` to not pass it
- Fixed: Simplified rate limit check in proxy.rs to use `server.rate_window_seconds.is_some()` instead of destructuring an unused binding

### spx-uiux-verifier
- Fixed: Added `aria-label` to rate limit badge in `GroupDetailPage.vue` for screen reader accessibility
