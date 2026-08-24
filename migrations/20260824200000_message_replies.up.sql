-- F037: single-message replies.

ALTER TABLE messages
    ADD COLUMN referenced_message_id UUID REFERENCES messages (id) ON DELETE SET NULL;

CREATE INDEX messages_referenced_message_id_idx
    ON messages (referenced_message_id)
    WHERE referenced_message_id IS NOT NULL;
