ALTER TABLE uptime_checks ADD COLUMN request_model TEXT;

CREATE INDEX IF NOT EXISTS idx_uptime_checks_group_model_time
    ON uptime_checks (group_id, request_model, created_at);
