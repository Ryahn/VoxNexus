-- F030: category/channel permission overrides (role or member subject).

CREATE TABLE permission_overrides (
    id UUID PRIMARY KEY,
    community_id UUID NOT NULL REFERENCES communities (id) ON DELETE CASCADE,
    channel_id UUID REFERENCES channels (id) ON DELETE CASCADE,
    category_id UUID REFERENCES categories (id) ON DELETE CASCADE,
    role_id UUID REFERENCES community_roles (id) ON DELETE CASCADE,
    account_id UUID REFERENCES accounts (id) ON DELETE CASCADE,
    permissions JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT permission_overrides_scope_check CHECK (
        (channel_id IS NOT NULL AND category_id IS NULL)
        OR (channel_id IS NULL AND category_id IS NOT NULL)
    ),
    CONSTRAINT permission_overrides_subject_check CHECK (
        (role_id IS NOT NULL AND account_id IS NULL)
        OR (role_id IS NULL AND account_id IS NOT NULL)
    )
);

CREATE UNIQUE INDEX permission_overrides_channel_role_unique
    ON permission_overrides (channel_id, role_id)
    WHERE channel_id IS NOT NULL AND role_id IS NOT NULL;

CREATE UNIQUE INDEX permission_overrides_channel_member_unique
    ON permission_overrides (channel_id, account_id)
    WHERE channel_id IS NOT NULL AND account_id IS NOT NULL;

CREATE UNIQUE INDEX permission_overrides_category_role_unique
    ON permission_overrides (category_id, role_id)
    WHERE category_id IS NOT NULL AND role_id IS NOT NULL;

CREATE UNIQUE INDEX permission_overrides_category_member_unique
    ON permission_overrides (category_id, account_id)
    WHERE category_id IS NOT NULL AND account_id IS NOT NULL;

CREATE INDEX permission_overrides_channel_idx ON permission_overrides (channel_id)
    WHERE channel_id IS NOT NULL;

CREATE INDEX permission_overrides_category_idx ON permission_overrides (category_id)
    WHERE category_id IS NOT NULL;
