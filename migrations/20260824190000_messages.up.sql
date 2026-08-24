-- F034: channel text messages.

CREATE TABLE messages (
    id UUID PRIMARY KEY,
    channel_id UUID NOT NULL REFERENCES channels (id) ON DELETE CASCADE,
    community_id UUID NOT NULL REFERENCES communities (id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES accounts (id) ON DELETE RESTRICT,
    content TEXT NOT NULL,
    nonce TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    edited_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT messages_content_len CHECK (
        char_length(content) >= 1 AND char_length(content) <= 4000
    )
);

CREATE UNIQUE INDEX messages_channel_author_nonce_unique
    ON messages (channel_id, author_id, nonce)
    WHERE nonce IS NOT NULL AND deleted_at IS NULL;

CREATE INDEX messages_channel_id_idx
    ON messages (channel_id, id DESC)
    WHERE deleted_at IS NULL;

-- Ensure @everyone can view and send in public text channels.
UPDATE community_roles
SET
    permissions = CASE
        WHEN permissions ? 'allow' THEN
            jsonb_set(
                permissions,
                '{allow,text}',
                to_jsonb(
                    COALESCE((permissions #>> '{allow,text}')::bigint, 0) | 3
                ),
                true
            )
        WHEN permissions ? 'families' THEN
            jsonb_build_object(
                'allow',
                jsonb_set(
                    COALESCE(permissions -> 'families', '{}'::jsonb),
                    '{text}',
                    to_jsonb(
                        COALESCE((permissions #>> '{families,text}')::bigint, 0) | 3
                    ),
                    true
                ),
                'deny',
                '{}'::jsonb
            )
        ELSE
            jsonb_build_object(
                'allow', jsonb_build_object('text', 3),
                'deny', '{}'::jsonb
            )
    END,
    updated_at = now()
WHERE is_everyone;
