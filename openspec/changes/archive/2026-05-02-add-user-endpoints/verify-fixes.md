## 2026-05-02 Round 1 (from apply auto-verify)

### Verifier
- Fixed: Removed the proxy's early empty `config.servers` 429 path so priority user endpoints are loaded and attempted even when no group servers are configured.
- Fixed: Routed bonus fallback TPM wait failures through `fallback_or_subscription_error` so fallback user endpoints can be attempted before returning a final rate-limit error.
- Fixed: Changed user endpoint create count-query handling to fail closed with HTTP 500 when the count query fails, preserving the max-10 limit.
- Fixed: Added targeted backend unit coverage for the endpoint limit constant, public key validation, model mapping JSON object validation, and priority mode validation.
