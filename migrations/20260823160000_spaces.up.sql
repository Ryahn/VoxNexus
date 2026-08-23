-- F022 Spaces (Guilded-style groups). Not nested — no parent_space_id.

CREATE TABLE spaces (
    id UUID PRIMARY KEY,
    community_id UUID NOT NULL REFERENCES communities (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    topic TEXT NOT NULL DEFAULT '',
    game TEXT NOT NULL DEFAULT '',
    visibility TEXT NOT NULL DEFAULT 'open'
        CHECK (visibility IN ('open', 'restricted')),
    icon_object_id UUID REFERENCES objects (id) ON DELETE SET NULL,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX spaces_community_id_idx ON spaces (community_id);
CREATE INDEX spaces_community_position_idx ON spaces (community_id, position);
