CREATE TABLE user_endpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_key_id UUID NOT NULL REFERENCES group_keys(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    api_key TEXT NOT NULL,
    model_mappings JSONB NOT NULL DEFAULT '{}'::jsonb,
    quota_url TEXT,
    quota_headers JSONB,
    priority_mode TEXT NOT NULL DEFAULT 'fallback' CHECK (priority_mode IN ('priority', 'fallback')),
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_user_endpoints_group_key_created_at ON user_endpoints(group_key_id, created_at);
CREATE INDEX idx_user_endpoints_group_key_enabled_mode_created_at ON user_endpoints(group_key_id, is_enabled, priority_mode, created_at);

ALTER TABLE token_usage_logs
    ADD COLUMN IF NOT EXISTS user_endpoint_id UUID;

CREATE INDEX idx_token_usage_logs_user_endpoint_id ON token_usage_logs(user_endpoint_id);
