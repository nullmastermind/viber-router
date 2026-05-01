ALTER TABLE settings
ADD COLUMN IF NOT EXISTS timezone TEXT NOT NULL DEFAULT 'Asia/Ho_Chi_Minh';

ALTER TABLE subscription_plans
ADD COLUMN IF NOT EXISTS weekly_cost_limit_usd FLOAT8;

ALTER TABLE key_subscriptions
ADD COLUMN IF NOT EXISTS weekly_cost_limit_usd FLOAT8;
