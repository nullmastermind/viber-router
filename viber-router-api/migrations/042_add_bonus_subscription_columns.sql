-- Add bonus subscription columns to key_subscriptions
ALTER TABLE key_subscriptions
    ADD COLUMN IF NOT EXISTS bonus_base_url TEXT,
    ADD COLUMN IF NOT EXISTS bonus_api_key TEXT,
    ADD COLUMN IF NOT EXISTS bonus_name TEXT,
    ADD COLUMN IF NOT EXISTS bonus_quota_url TEXT,
    ADD COLUMN IF NOT EXISTS bonus_quota_headers JSONB;

-- Update sub_type CHECK constraint on key_subscriptions to include 'bonus'
ALTER TABLE key_subscriptions DROP CONSTRAINT IF EXISTS key_subscriptions_sub_type_check;
ALTER TABLE key_subscriptions ADD CONSTRAINT key_subscriptions_sub_type_check
    CHECK (sub_type IN ('fixed', 'hourly_reset', 'pay_per_request', 'bonus'));

-- Update sub_type CHECK constraint on subscription_plans to include 'bonus'
ALTER TABLE subscription_plans DROP CONSTRAINT IF EXISTS subscription_plans_sub_type_check;
ALTER TABLE subscription_plans ADD CONSTRAINT subscription_plans_sub_type_check
    CHECK (sub_type IN ('fixed', 'hourly_reset', 'pay_per_request', 'bonus'));
