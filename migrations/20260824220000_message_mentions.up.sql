-- F039: structured message mentions for inbox (F043) and authz.

CREATE TABLE message_mentions (
    message_id UUID NOT NULL REFERENCES messages (id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    target_id UUID,
    CONSTRAINT message_mentions_kind_check CHECK (
        kind IN ('user', 'role', 'everyone', 'here')
    ),
    CONSTRAINT message_mentions_target_check CHECK (
        (kind IN ('user', 'role') AND target_id IS NOT NULL)
        OR (kind IN ('everyone', 'here') AND target_id IS NULL)
    )
);

CREATE UNIQUE INDEX message_mentions_unique_idx
    ON message_mentions (
        message_id,
        kind,
        COALESCE(target_id, '00000000-0000-0000-0000-000000000000'::uuid)
    );

CREATE INDEX message_mentions_target_idx
    ON message_mentions (kind, target_id)
    WHERE target_id IS NOT NULL;

-- Grant text.mention_roles (1 << 6 = 64) on @everyone.
UPDATE community_roles
SET
    permissions = CASE
        WHEN permissions ? 'allow' THEN
            jsonb_set(
                permissions,
                '{allow,text}',
                to_jsonb(
                    COALESCE((permissions #>> '{allow,text}')::bigint, 0) | 64
                ),
                true
            )
        ELSE permissions
    END,
    updated_at = now()
WHERE is_everyone;
