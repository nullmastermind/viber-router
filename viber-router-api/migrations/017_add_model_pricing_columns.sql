-- Add pricing columns to models table (USD per 1M tokens)
ALTER TABLE models ADD COLUMN input_1m_usd NUMERIC;
ALTER TABLE models ADD COLUMN output_1m_usd NUMERIC;
ALTER TABLE models ADD COLUMN cache_write_1m_usd NUMERIC;
ALTER TABLE models ADD COLUMN cache_read_1m_usd NUMERIC;
