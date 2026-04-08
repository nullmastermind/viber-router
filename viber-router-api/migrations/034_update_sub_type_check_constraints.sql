ALTER TABLE subscription_plans
    DROP CONSTRAINT IF EXISTS subscription_plans_sub_type_check,
    ADD CONSTRAINT subscription_plans_sub_type_check
        CHECK (sub_type IN ('fixed', 'hourly_reset', 'pay_per_request'));

ALTER TABLE key_subscriptions
    DROP CONSTRAINT IF EXISTS key_subscriptions_sub_type_check,
    ADD CONSTRAINT key_subscriptions_sub_type_check
        CHECK (sub_type IN ('fixed', 'hourly_reset', 'pay_per_request'));
