ALTER TABLE group_servers
ADD COLUMN IF NOT EXISTS per_key_max_requests INT,
ADD COLUMN IF NOT EXISTS per_key_rate_window_seconds INT;
