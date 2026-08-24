-- F030-A Roles v2: weight, groups, cosmetics, display order.

CREATE TABLE community_role_groups (
    id UUID PRIMARY KEY,
    community_id UUID NOT NULL REFERENCES communities (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX community_role_groups_name_idx
    ON community_role_groups (community_id, name);
CREATE INDEX community_role_groups_community_order_idx
    ON community_role_groups (community_id, display_order);

ALTER TABLE community_roles
    ADD COLUMN weight INTEGER NOT NULL DEFAULT 1000,
    ADD COLUMN group_id UUID REFERENCES community_role_groups (id) ON DELETE SET NULL,
    ADD COLUMN short_tag TEXT NOT NULL DEFAULT '',
    ADD COLUMN icon_emoji TEXT,
    ADD COLUMN icon_object_key TEXT,
    ADD COLUMN gradient TEXT,
    ADD COLUMN role_card JSONB NOT NULL DEFAULT '{}'::jsonb;

-- @everyone stays lowest priority (highest weight). Custom roles get unique weights by position.
UPDATE community_roles
SET weight = 1000
WHERE is_everyone;

UPDATE community_roles r
SET weight = GREATEST(1, 999 - r.position)
WHERE NOT r.is_everyone;

-- Resolve any residual collisions within a community (unlikely after position map).
WITH ranked AS (
    SELECT
        id,
        community_id,
        ROW_NUMBER() OVER (
            PARTITION BY community_id
            ORDER BY is_everyone DESC, weight ASC, position ASC, created_at ASC
        ) AS rn
    FROM community_roles
    WHERE NOT is_everyone
)
UPDATE community_roles r
SET weight = GREATEST(1, LEAST(999, ranked.rn))
FROM ranked
WHERE r.id = ranked.id;

CREATE UNIQUE INDEX community_roles_weight_idx ON community_roles (community_id, weight);
CREATE INDEX community_roles_group_idx ON community_roles (group_id);

-- Normalize legacy permissions to tri-state { allow, deny }.
UPDATE community_roles
SET permissions = jsonb_build_object(
    'allow',
    COALESCE(
        CASE
            WHEN permissions ? 'allow' THEN permissions->'allow'
            WHEN permissions ? 'families' THEN permissions->'families'
            ELSE '{}'::jsonb
        END,
        '{}'::jsonb
    ) || CASE
        WHEN COALESCE((permissions->>'manage_roles')::boolean, FALSE)
            THEN jsonb_build_object('community', 4)
        ELSE '{}'::jsonb
    END,
    'deny',
    COALESCE(permissions->'deny', '{}'::jsonb)
)
WHERE NOT (permissions ? 'allow' AND permissions ? 'deny');
