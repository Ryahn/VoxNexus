DROP INDEX IF EXISTS communities_instance_invite_path_unique;

ALTER TABLE communities
    DROP COLUMN IF EXISTS invite_path,
    DROP COLUMN IF EXISTS invite_splash_object_id,
    DROP COLUMN IF EXISTS tag_badge_object_id,
    DROP COLUMN IF EXISTS tag_color,
    DROP COLUMN IF EXISTS tag_name;
