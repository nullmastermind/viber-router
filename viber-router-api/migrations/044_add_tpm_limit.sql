ALTER TABLE subscription_plans
ADD COLUMN IF NOT EXISTS tpm_limit FLOAT8;

ALTER TABLE key_subscriptions
ADD COLUMN IF NOT EXISTS tpm_limit FLOAT8;
