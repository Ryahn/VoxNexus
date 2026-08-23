-- F017 singleton instance settings row.

CREATE TABLE instances (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    public_url TEXT NOT NULL,
    registration_mode TEXT NOT NULL,
    community_creation_mode TEXT NOT NULL,
    oidc_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    oidc_issuer TEXT,
    oidc_client_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT instances_registration_mode_check
        CHECK (registration_mode IN ('open', 'invite', 'closed')),
    CONSTRAINT instances_community_creation_mode_check
        CHECK (community_creation_mode IN ('open', 'admin_only', 'single'))
);

INSERT INTO instances (
    id,
    name,
    public_url,
    registration_mode,
    community_creation_mode,
    oidc_enabled,
    created_at,
    updated_at
) VALUES (
    '01900000-0000-7000-8000-000000000001',
    'VoxNexus',
    'http://127.0.0.1:8080',
    'open',
    'open',
    FALSE,
    now(),
    now()
);
