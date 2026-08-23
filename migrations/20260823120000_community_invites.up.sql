-- F021 community invite links.

CREATE TABLE community_invites (
    id UUID PRIMARY KEY,
    community_id UUID NOT NULL REFERENCES communities (id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    max_uses INTEGER CHECK (max_uses IS NULL OR max_uses > 0),
    uses INTEGER NOT NULL DEFAULT 0 CHECK (uses >= 0),
    expires_at TIMESTAMPTZ,
    paused BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT community_invites_code_unique UNIQUE (code)
);

CREATE INDEX community_invites_community_id_idx ON community_invites (community_id);
CREATE INDEX community_invites_created_by_idx ON community_invites (created_by);
