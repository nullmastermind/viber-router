ALTER TABLE token_usage_logs ADD COLUMN IF NOT EXISTS cost_usd FLOAT8;
ALTER TABLE token_usage_logs ADD COLUMN IF NOT EXISTS subscription_id UUID;
