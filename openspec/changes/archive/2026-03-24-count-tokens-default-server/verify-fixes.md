## [2026-03-24] Round 1 (from spx-apply auto-verify)

### spx-arch-verifier
- Fixed: Removed no-op `ctServerOptions` computed wrapper in GroupDetailPage.vue; template now binds directly to `allServers`

### Not fixed (architectural notes for future)
- DRY violation in proxy.rs: count-tokens default server block (~110 lines) duplicates waterfall loop request-building logic. Extracting a shared `attempt_upstream` function would reduce this but is a significant refactor best done as a separate change.
- Latent `is_last_server` miscalculation when ct server is last in failover chain: unreachable in practice (count_tokens is non-streaming, TTFT branch never entered). Would need fixing if skip pattern is reused for streaming endpoints.
