CREATE TABLE IF NOT EXISTS proxy_logs (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    group_id UUID NOT NULL,
    group_api_key TEXT NOT NULL,
    server_id UUID NOT NULL,
    server_name TEXT NOT NULL,
    request_path TEXT NOT NULL,
    request_method TEXT NOT NULL,
    status_code SMALLINT NOT NULL,
    error_type TEXT NOT NULL,
    latency_ms INTEGER NOT NULL,
    failover_chain JSONB NOT NULL DEFAULT '[]'::jsonb,
    request_model TEXT,
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

CREATE INDEX IF NOT EXISTS idx_proxy_logs_created_at ON proxy_logs (created_at);
CREATE INDEX IF NOT EXISTS idx_proxy_logs_group_id ON proxy_logs (group_id);
CREATE INDEX IF NOT EXISTS idx_proxy_logs_status_code ON proxy_logs (status_code);
CREATE INDEX IF NOT EXISTS idx_proxy_logs_group_api_key ON proxy_logs (group_api_key);
