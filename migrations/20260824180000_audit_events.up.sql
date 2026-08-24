-- F033: append-only community audit events.

CREATE TABLE audit_events (
    id UUID PRIMARY KEY,
    community_id UUID NOT NULL REFERENCES communities (id) ON DELETE CASCADE,
    actor_account_id UUID REFERENCES accounts (id) ON DELETE SET NULL,
    action TEXT NOT NULL,
    space_id UUID REFERENCES spaces (id) ON DELETE SET NULL,
    target_type TEXT,
    target_id UUID,
    summary TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX audit_events_community_created_idx
    ON audit_events (community_id, created_at DESC, id DESC);

CREATE INDEX audit_events_community_actor_idx
    ON audit_events (community_id, actor_account_id, created_at DESC)
    WHERE actor_account_id IS NOT NULL;

CREATE INDEX audit_events_community_action_idx
    ON audit_events (community_id, action, created_at DESC);

CREATE INDEX audit_events_community_space_idx
    ON audit_events (community_id, space_id, created_at DESC)
    WHERE space_id IS NOT NULL;
