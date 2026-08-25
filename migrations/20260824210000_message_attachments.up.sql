-- F038: message attachments (object metadata + pending upload → message bind).

ALTER TABLE messages DROP CONSTRAINT messages_content_len;
ALTER TABLE messages ADD CONSTRAINT messages_content_len CHECK (char_length(content) <= 4000);

CREATE TABLE message_attachments (
    id UUID PRIMARY KEY,
    message_id UUID REFERENCES messages (id) ON DELETE CASCADE,
    channel_id UUID NOT NULL REFERENCES channels (id) ON DELETE CASCADE,
    community_id UUID NOT NULL REFERENCES communities (id) ON DELETE CASCADE,
    object_id UUID NOT NULL REFERENCES objects (id) ON DELETE RESTRICT,
    thumbnail_object_id UUID REFERENCES objects (id) ON DELETE SET NULL,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    byte_size BIGINT NOT NULL,
    width INT,
    height INT,
    created_by UUID NOT NULL REFERENCES accounts (id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT message_attachments_object_id_unique UNIQUE (object_id),
    CONSTRAINT message_attachments_filename_len CHECK (
        char_length(filename) >= 1 AND char_length(filename) <= 255
    ),
    CONSTRAINT message_attachments_byte_size_positive CHECK (byte_size > 0)
);

CREATE INDEX message_attachments_message_id_idx
    ON message_attachments (message_id)
    WHERE message_id IS NOT NULL;

CREATE INDEX message_attachments_channel_pending_idx
    ON message_attachments (channel_id, created_by, created_at)
    WHERE message_id IS NULL;

-- Grant text.attach (1 << 2) on @everyone alongside view|send.
UPDATE community_roles
SET
    permissions = CASE
        WHEN permissions ? 'allow' THEN
            jsonb_set(
                permissions,
                '{allow,text}',
                to_jsonb(
                    COALESCE((permissions #>> '{allow,text}')::bigint, 0) | 7
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
                        COALESCE((permissions #>> '{families,text}')::bigint, 0) | 7
                    ),
                    true
                ),
                'deny',
                '{}'::jsonb
            )
        ELSE
            jsonb_build_object(
                'allow', jsonb_build_object('text', 7),
                'deny', '{}'::jsonb
            )
    END,
    updated_at = now()
WHERE is_everyone;
