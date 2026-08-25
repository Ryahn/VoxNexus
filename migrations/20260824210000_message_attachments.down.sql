UPDATE community_roles
SET
    permissions = CASE
        WHEN permissions ? 'allow' THEN
            jsonb_set(
                permissions,
                '{allow,text}',
                to_jsonb(
                    COALESCE((permissions #>> '{allow,text}')::bigint, 0) & ~4
                ),
                true
            )
        ELSE permissions
    END,
    updated_at = now()
WHERE is_everyone;

DROP TABLE IF EXISTS message_attachments;

ALTER TABLE messages DROP CONSTRAINT messages_content_len;
ALTER TABLE messages ADD CONSTRAINT messages_content_len CHECK (
    char_length(content) >= 1 AND char_length(content) <= 4000
);
