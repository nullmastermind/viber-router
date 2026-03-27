## Why

Sub-key holders (end users of the proxy) currently have no way to check their own usage or subscription status — only admins can see this data through the admin UI. A public, unauthenticated page that accepts a sub-key and displays its usage and subscription info lets users self-serve without admin involvement.

## What Changes

- New public backend endpoint `GET /api/public/usage?key=<sub-key>` that validates the sub-key and returns usage + subscription data (no admin auth required)
- IP-based rate limiting (30 req/min) on the public endpoint to prevent brute-force key scanning
- Token usage aggregated by model only — server info is excluded to hide upstream sources
- Subscription data includes cost_used and window reset time for hourly_reset subscriptions
- New frontend page at `/usage` (with key input form) and `/usage/:key` (direct link) — outside the admin layout, no login required
- Navigation guard updated to exempt `/usage` paths from admin token check

## Capabilities

### New Capabilities
- `public-usage-api`: Public backend endpoint for sub-key usage and subscription data, with IP-based rate limiting
- `public-usage-page`: Frontend page for viewing sub-key usage and subscriptions without admin login

### Modified Capabilities
<!-- No existing spec-level requirements are changing -->

## Impact

- **Backend**: New route module `routes/public/` with usage handler; new IP rate limit function in `rate_limiter.rs`; router registration in `routes/mod.rs`
- **Frontend**: New `PublicUsagePage.vue`; route additions in `routes.ts`; auth guard exemption in `router/index.ts`
- **APIs**: New public endpoint `GET /api/public/usage` — no breaking changes to existing admin APIs
- **Dependencies**: No new crate or npm dependencies required
