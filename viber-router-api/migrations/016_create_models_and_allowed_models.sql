-- Models master table
CREATE TABLE models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Group allowed models junction table
CREATE TABLE group_allowed_models (
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (group_id, model_id)
);

CREATE INDEX idx_group_allowed_models_model_id ON group_allowed_models(model_id);

-- Group key allowed models junction table
CREATE TABLE group_key_allowed_models (
    group_key_id UUID NOT NULL REFERENCES group_keys(id) ON DELETE CASCADE,
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (group_key_id, model_id)
);

CREATE INDEX idx_group_key_allowed_models_model_id ON group_key_allowed_models(model_id);
