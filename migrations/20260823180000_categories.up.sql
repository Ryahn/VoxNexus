-- F026 Channel categories (group channels in sidebar). Optional space scope.

CREATE TABLE categories (
    id UUID PRIMARY KEY,
    community_id UUID NOT NULL REFERENCES communities (id) ON DELETE CASCADE,
    space_id UUID REFERENCES spaces (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX categories_community_id_idx ON categories (community_id);
CREATE INDEX categories_space_id_idx ON categories (space_id);
CREATE INDEX categories_community_space_position_idx
    ON categories (community_id, space_id, position);
