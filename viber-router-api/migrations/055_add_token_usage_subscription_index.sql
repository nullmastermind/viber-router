-- Speed up the public /usage page aggregate over the last 30 days for bonus subscriptions.
-- The group_key_id index already exists (migration 014); only subscription_id was missing.
CREATE INDEX IF NOT EXISTS idx_token_usage_logs_subscription_created
    ON token_usage_logs (subscription_id, created_at);
