-- F029 Default text.view on @everyone for existing communities.

UPDATE community_roles
SET permissions = '{"families":{"text":1}}'::jsonb,
    updated_at = now()
WHERE is_everyone
  AND (permissions = '{}'::jsonb OR permissions = 'null'::jsonb);
