ALTER TABLE ttft_logs ADD COLUMN IF NOT EXISTS group_key_id UUID;

CREATE INDEX IF NOT EXISTS idx_ttft_logs_group_key_id_created
    ON ttft_logs (group_key_id, created_at);
