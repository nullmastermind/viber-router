ALTER TABLE group_servers
    ADD COLUMN retry_status_codes INTEGER[] DEFAULT NULL,
    ADD COLUMN retry_count INTEGER DEFAULT NULL,
    ADD COLUMN retry_delay_seconds DOUBLE PRECISION DEFAULT NULL;
