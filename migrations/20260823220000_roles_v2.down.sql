-- F030-A Roles v2 down.

DROP INDEX IF EXISTS community_roles_group_idx;
DROP INDEX IF EXISTS community_roles_weight_idx;

ALTER TABLE community_roles
    DROP COLUMN IF EXISTS role_card,
    DROP COLUMN IF EXISTS gradient,
    DROP COLUMN IF EXISTS icon_object_key,
    DROP COLUMN IF EXISTS icon_emoji,
    DROP COLUMN IF EXISTS short_tag,
    DROP COLUMN IF EXISTS group_id,
    DROP COLUMN IF EXISTS weight;

DROP TABLE IF EXISTS community_role_groups;
