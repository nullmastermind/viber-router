## [2026-03-26] Round 1 (from spx-apply auto-verify)

### spx-verifier
- Fixed: [CRITICAL] SETNX loser in `ensure_activated` — added comment clarifying that caller re-fetches from DB to get stored `activated_at`; the DB write from the winner is visible to subsequent reads
- Noted: [CRITICAL] "Used" column missing from subscriptions table — requires backend endpoint to return current cost per subscription; deferred to follow-up

### spx-arch-verifier
- Fixed: [CRITICAL] `ensure_activated` Redis unavailable — added DB-only fallback path with warning log when Redis is down
- Fixed: [CRITICAL] `sub_activated:{sub_id}` Redis key missing TTL — added EX with duration_days + 1h buffer
- Fixed: [CRITICAL] `update_cost_counters` silent drop — added tracing::warn log when Redis unavailable
- Fixed: [WARNING] `key_subscriptions.sub_type` missing CHECK constraint — added CHECK (sub_type IN ('fixed', 'hourly_reset'))
