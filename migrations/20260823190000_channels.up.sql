-- F027 Channel framework (shell — no messages yet).

CREATE TABLE channels (
    id UUID PRIMARY KEY,
    community_id UUID NOT NULL REFERENCES communities (id) ON DELETE CASCADE,
    space_id UUID REFERENCES spaces (id) ON DELETE CASCADE,
    category_id UUID REFERENCES categories (id) ON DELETE SET NULL,
    channel_type TEXT NOT NULL
        CHECK (
            channel_type IN (
                'text',
                'voice',
                'forum',
                'announcement',
                'calendar',
                'scheduling',
                'docs',
                'tasks',
                'media',
                'stage',
                'streaming'
            )
        ),
    name TEXT NOT NULL,
    topic TEXT NOT NULL DEFAULT '',
    position INTEGER NOT NULL DEFAULT 0,
    archived_at TIMESTAMPTZ,
    config JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX channels_community_id_idx ON channels (community_id);
CREATE INDEX channels_space_id_idx ON channels (space_id);
CREATE INDEX channels_category_id_idx ON channels (category_id);
CREATE INDEX channels_scope_position_idx ON channels (community_id, space_id, category_id, position);
