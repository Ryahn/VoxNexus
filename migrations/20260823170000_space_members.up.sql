-- F023 Space membership. Role allow-lists wait for F028 roles.

CREATE TABLE space_members (
    space_id UUID NOT NULL REFERENCES spaces (id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (space_id, account_id)
);

CREATE INDEX space_members_account_id_idx ON space_members (account_id);
