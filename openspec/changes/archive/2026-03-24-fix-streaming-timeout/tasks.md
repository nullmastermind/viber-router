## 1. Configure HTTP Client

- [x] 1.1 In `viber-router-api/src/main.rs`, replace `reqwest::Client::new()` with `reqwest::Client::builder()` configured with: `timeout(8h)`, `connect_timeout(10s)`, `pool_idle_timeout(1h)`, `tcp_keepalive(30s)`, `tcp_nodelay(true)`, `http2_keep_alive_interval(30s)`, `http2_keep_alive_timeout(10s)`, `http2_keep_alive_while_idle(true)` ← (verify: all settings from design.md are present, `use std::time::Duration` is imported, compiles without errors)

## 2. Validate

- [x] 2.1 Run `just check` — ensure no type errors or lint warnings ← (verify: clean output from cargo check + cargo clippy + biome)
