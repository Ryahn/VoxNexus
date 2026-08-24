-- F019-A: server tag, invite splash, custom invite path (always-on cosmetics).

ALTER TABLE communities
    ADD COLUMN tag_name TEXT NOT NULL DEFAULT '',
    ADD COLUMN tag_color TEXT NOT NULL DEFAULT '',
    ADD COLUMN tag_badge_object_id UUID REFERENCES objects (id) ON DELETE SET NULL,
    ADD COLUMN invite_splash_object_id UUID REFERENCES objects (id) ON DELETE SET NULL,
    ADD COLUMN invite_path TEXT;

CREATE UNIQUE INDEX communities_instance_invite_path_unique
    ON communities (instance_id, invite_path)
    WHERE invite_path IS NOT NULL AND invite_path <> '';
