-- F037: drop reply column.

ALTER TABLE messages
    DROP COLUMN IF EXISTS referenced_message_id;
