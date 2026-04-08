ALTER TABLE subscription_plans
    ADD COLUMN IF NOT EXISTS model_request_costs JSONB NOT NULL DEFAULT '{}';

ALTER TABLE key_subscriptions
    ADD COLUMN IF NOT EXISTS model_request_costs JSONB NOT NULL DEFAULT '{}';
