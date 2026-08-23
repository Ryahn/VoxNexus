-- F019 communities and membership (creator = owner).

CREATE TABLE communities (
    id UUID PRIMARY KEY,
    instance_id UUID NOT NULL REFERENCES instances (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    timezone TEXT NOT NULL DEFAULT 'UTC',
    join_mode TEXT NOT NULL DEFAULT 'open'
        CHECK (join_mode IN ('open', 'invite', 'application')),
    owner_account_id UUID NOT NULL REFERENCES accounts (id) ON DELETE RESTRICT,
    icon_object_id UUID REFERENCES objects (id) ON DELETE SET NULL,
    banner_object_id UUID REFERENCES objects (id) ON DELETE SET NULL,
    discoverable_on_instance BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT communities_instance_slug_unique UNIQUE (instance_id, slug)
);

CREATE INDEX communities_instance_id_idx ON communities (instance_id);
CREATE INDEX communities_owner_account_id_idx ON communities (owner_account_id);

CREATE TABLE community_members (
    community_id UUID NOT NULL REFERENCES communities (id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'member'
        CHECK (role IN ('owner', 'member')),
    nickname TEXT NOT NULL DEFAULT '',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (community_id, account_id)
);

CREATE INDEX community_members_account_id_idx ON community_members (account_id);
