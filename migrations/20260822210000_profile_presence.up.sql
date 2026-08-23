-- F018 presence preference and custom status on profiles.

ALTER TABLE profiles
    ADD COLUMN presence_status TEXT NOT NULL DEFAULT 'online',
    ADD COLUMN custom_status TEXT NOT NULL DEFAULT '';

ALTER TABLE profiles
    ADD CONSTRAINT profiles_presence_status_check
        CHECK (presence_status IN ('online', 'idle', 'dnd', 'invisible'));
