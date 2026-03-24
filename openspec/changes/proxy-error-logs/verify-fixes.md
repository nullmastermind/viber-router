## [2026-03-24] Round 1 (from spx-apply auto-verify)

### spx-arch-verifier
- Fixed: [CRITICAL] Graceful shutdown now drains log buffer — stored JoinHandle, drop state after server stops, await flush task in main.rs
- Fixed: [CRITICAL] SQL identifier quoting in partition.rs — table names in CREATE/DROP DDL now quoted with double quotes to prevent injection
