-- Add rate multiplier columns to group_servers table
ALTER TABLE group_servers ADD COLUMN rate_input FLOAT8;
ALTER TABLE group_servers ADD COLUMN rate_output FLOAT8;
ALTER TABLE group_servers ADD COLUMN rate_cache_write FLOAT8;
ALTER TABLE group_servers ADD COLUMN rate_cache_read FLOAT8;
