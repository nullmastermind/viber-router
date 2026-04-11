CREATE TABLE group_user_agents (
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_agent TEXT NOT NULL,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (group_id, user_agent)
);

CREATE TABLE group_blocked_user_agents (
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_agent TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (group_id, user_agent)
);

CREATE INDEX idx_group_blocked_user_agents_group_id ON group_blocked_user_agents(group_id);
