-- Add TTFT timeout configuration to groups
ALTER TABLE groups ADD COLUMN IF NOT EXISTS ttft_timeout_ms INTEGER;

-- Create partitioned ttft_logs table
CREATE TABLE IF NOT EXISTS ttft_logs (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    group_id UUID NOT NULL,
    server_id UUID NOT NULL,
    request_model TEXT,
    ttft_ms INTEGER,
    timed_out BOOLEAN NOT NULL DEFAULT false,
    request_path TEXT NOT NULL,
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

CREATE INDEX IF NOT EXISTS idx_ttft_logs_group_server_created
    ON ttft_logs (group_id, server_id, created_at);
