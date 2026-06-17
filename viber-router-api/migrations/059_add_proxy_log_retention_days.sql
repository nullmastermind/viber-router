-- Max age (in days) of proxy logs shown on the /#/logs page before the daily
-- midnight purge removes them. Default 3 days. Separate from the coarse
-- LOG_RETENTION_DAYS partition cleanup (default 30) which acts as a safety net.
ALTER TABLE settings
    ADD COLUMN IF NOT EXISTS proxy_log_retention_days INT NOT NULL DEFAULT 3;
