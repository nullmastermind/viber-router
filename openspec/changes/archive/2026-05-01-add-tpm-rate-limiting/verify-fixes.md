## 2026-05-01 Round 1 (from apply auto-verify)

### Verifier
- Fixed: Deferred fallback subscription TPM wait and RPM increment in `/root/projects/viber-router/viber-router-api/src/routes/proxy.rs` until after all bonus servers fail, so fallback limits no longer block bonus routing before bonus servers are attempted.
- Fixed: Passed the selected subscription TPM limit into streaming usage tracking in `/root/projects/viber-router/viber-router-api/src/routes/proxy.rs` and guarded streaming TPM increments on `tpm_limit.is_some()`, keeping bonus subscriptions out of TPM accounting.
- Fixed: Guarded non-streaming group waterfall TPM increments on the selected subscription having a TPM limit, matching nullable TPM semantics and avoiding increments for subscriptions without TPM enforcement.
