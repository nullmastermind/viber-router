CREATE TABLE IF NOT EXISTS subscription_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    sub_type TEXT NOT NULL CHECK (sub_type IN ('fixed', 'hourly_reset')),
    cost_limit_usd FLOAT8 NOT NULL,
    model_limits JSONB NOT NULL DEFAULT '{}',
    reset_hours INT,
    duration_days INT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
