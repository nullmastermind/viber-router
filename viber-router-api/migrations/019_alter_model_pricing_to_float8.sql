-- Change pricing columns from NUMERIC to FLOAT8 for sqlx f64 compatibility
ALTER TABLE models ALTER COLUMN input_1m_usd TYPE FLOAT8;
ALTER TABLE models ALTER COLUMN output_1m_usd TYPE FLOAT8;
ALTER TABLE models ALTER COLUMN cache_write_1m_usd TYPE FLOAT8;
ALTER TABLE models ALTER COLUMN cache_read_1m_usd TYPE FLOAT8;
