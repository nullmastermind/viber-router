## [2026-03-24] Round 1 (from spx-apply auto-verify)

### spx-verifier
- Fixed: Chart datasets not aligned to shared label array — rewrote `ttftChartData` computed to build `uniqueTimes` first, then map each server's data points against the shared labels using Map/Set lookups (GroupDetailPage.vue)
- Fixed: Task 9.3 marked as deferred (manual integration test requiring running server + mock upstream)

### spx-arch-verifier
- Fixed: N+1 query in ttft.rs — replaced per-server data point queries with a single bulk query, then grouped in Rust using HashMap (routes/admin/ttft.rs)
- Fixed: SQL interval interpolation pattern — extracted `resolve_interval()` function that returns `&'static str`, making the safety invariant explicit and compiler-enforced (routes/admin/ttft.rs)
- Fixed: `AggRow.timeout_count` and `total_count` changed from `Option<i64>` to `i64` since COUNT(*) never returns NULL (routes/admin/ttft.rs)
- Fixed: Used `enumerate()` on server loop instead of `position()` lookup for O(1) server index (routes/proxy.rs)

### spx-uiux-verifier
- Fixed: Added `ttftLoading` ref with spinner shown during initial TTFT data fetch (GroupDetailPage.vue)
- Fixed: Added `ttftError` ref with q-banner error display when TTFT fetch fails (GroupDetailPage.vue)
