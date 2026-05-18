ALTER TABLE settings
ADD COLUMN IF NOT EXISTS openai_compat_base_url TEXT;
