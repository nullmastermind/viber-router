ALTER TABLE groups ADD COLUMN count_tokens_server_id UUID REFERENCES servers(id) ON DELETE SET NULL;
ALTER TABLE groups ADD COLUMN count_tokens_model_mappings JSONB NOT NULL DEFAULT '{}';
