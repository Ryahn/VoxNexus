-- F014 profiles and object metadata for avatars/banners.

CREATE TABLE objects (
    id UUID PRIMARY KEY,
    storage_key TEXT NOT NULL,
    sha256 BYTEA NOT NULL,
    mime TEXT NOT NULL,
    byte_size BIGINT NOT NULL,
    created_by UUID NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT objects_storage_key_unique UNIQUE (storage_key)
);

CREATE INDEX objects_created_by_idx ON objects (created_by);

CREATE TABLE profiles (
    account_id UUID PRIMARY KEY REFERENCES accounts (id) ON DELETE CASCADE,
    display_name TEXT NOT NULL DEFAULT '',
    bio TEXT NOT NULL DEFAULT '',
    avatar_object_id UUID REFERENCES objects (id) ON DELETE SET NULL,
    banner_object_id UUID REFERENCES objects (id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Backfill empty profiles for accounts created before this migration.
INSERT INTO profiles (account_id)
SELECT id FROM accounts
ON CONFLICT (account_id) DO NOTHING;
