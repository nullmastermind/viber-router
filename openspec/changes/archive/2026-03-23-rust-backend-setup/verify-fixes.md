## [2026-03-23] Round 1 (from spx-apply auto-verify)

### spx-verifier
- Fixed: Removed direct `redis` dependency from Cargo.toml — `deadpool-redis` re-exports it, direct dep was phantom
- Fixed: Made `.env.example` use placeholder credentials instead of real-looking defaults

### spx-arch-verifier
- Fixed: Removed phantom `redis` direct dependency (same as above, flagged by both verifiers)
- Skipped: `shutdown_signal` `.expect()` — this is idiomatic for signal handlers in `with_graceful_shutdown` which requires `Future<Output = ()>`; panic on signal registration failure is acceptable
- Skipped: `config.rust_log` field — provides a clean single-source config pattern even if `EnvFilter` also reads env; removing it would make the config struct incomplete
