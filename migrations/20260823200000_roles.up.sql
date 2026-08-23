-- F028 Community roles and assignments.

CREATE TABLE community_roles (
    id UUID PRIMARY KEY,
    community_id UUID NOT NULL REFERENCES communities (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    color TEXT NOT NULL DEFAULT '141 152 173',
    hoist BOOLEAN NOT NULL DEFAULT FALSE,
    mentionable BOOLEAN NOT NULL DEFAULT FALSE,
    permissions JSONB NOT NULL DEFAULT '{}',
    is_everyone BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX community_roles_everyone_idx ON community_roles (community_id)
    WHERE is_everyone;
CREATE UNIQUE INDEX community_roles_name_idx ON community_roles (community_id, name);
CREATE INDEX community_roles_community_position_idx ON community_roles (community_id, position);

CREATE TABLE community_role_assignments (
    community_id UUID NOT NULL REFERENCES communities (id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES community_roles (id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, role_id)
);

CREATE INDEX community_role_assignments_community_idx ON community_role_assignments (community_id);
CREATE INDEX community_role_assignments_role_idx ON community_role_assignments (role_id);

INSERT INTO community_roles (
    id, community_id, name, position, color, hoist, mentionable, permissions,
    is_everyone, created_at, updated_at
)
SELECT
    gen_random_uuid(),
    c.id,
    '@everyone',
    0,
    '141 152 173',
    FALSE,
    FALSE,
    '{}',
    TRUE,
    c.created_at,
    c.updated_at
FROM communities c
WHERE NOT EXISTS (
    SELECT 1 FROM community_roles r
    WHERE r.community_id = c.id AND r.is_everyone
);
