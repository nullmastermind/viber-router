CREATE TABLE IF NOT EXISTS key_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_key_id UUID NOT NULL REFERENCES group_keys(id) ON DELETE CASCADE,
    plan_id UUID REFERENCES subscription_plans(id),
    sub_type TEXT NOT NULL CHECK (sub_type IN ('fixed', 'hourly_reset')),
    cost_limit_usd FLOAT8 NOT NULL,
    model_limits JSONB NOT NULL DEFAULT '{}',
    reset_hours INT,
    duration_days INT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    activated_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_key_subscriptions_key_status ON key_subscriptions (group_key_id, status);
