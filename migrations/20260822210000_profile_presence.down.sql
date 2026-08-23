ALTER TABLE profiles DROP CONSTRAINT IF EXISTS profiles_presence_status_check;
ALTER TABLE profiles DROP COLUMN IF EXISTS custom_status;
ALTER TABLE profiles DROP COLUMN IF EXISTS presence_status;
