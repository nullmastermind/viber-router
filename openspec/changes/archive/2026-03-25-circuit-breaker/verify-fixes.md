## [2026-03-25] Round 1 (from spx-apply auto-verify)

### spx-verifier
- Fixed: `record_error` in `circuit_breaker.rs` now uses `SET NX` for `cb:open` key to prevent TTL reset when circuit is already open; only returns true (trigger alert) when key was newly set

### spx-arch-verifier
- Fixed: Moved `validate_cb_fields` from `models/group_server.rs` to `routes/admin/group_servers.rs` to remove Axum dependency from model layer
- Fixed: `check_re_enabled` now uses a marker key set at circuit-open time (in `record_error`) instead of `SET NX` on first check, preventing phantom re-enable alerts for servers that were never tripped

### spx-uiux-verifier
- Fixed: Circuit-open badge color changed from raw `"red"` to Quasar semantic token `"negative"`
- Fixed: Added pre-save validation in `onSaveEditServer` to check all-or-nothing CB fields before API call
- Fixed: Added `min="1"` to all three CB number inputs in edit dialog
