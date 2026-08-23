-- Profiles created before we seeded display_name from email local-part.
UPDATE profiles AS p
SET display_name = left(split_part(a.email, '@', 1), 64),
    updated_at = now()
FROM accounts AS a
WHERE p.account_id = a.id
  AND p.display_name = ''
  AND a.email <> '';
