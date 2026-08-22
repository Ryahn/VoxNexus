-- F011 accounts + F012 sessions. Local auth and future OIDC identities.

CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    instance_id UUID NOT NULL,
    email TEXT,
    password_hash TEXT,
    is_bot BOOLEAN NOT NULL DEFAULT FALSE,
    is_instance_admin BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT accounts_email_unique UNIQUE (email)
);

CREATE TABLE auth_identities (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    issuer TEXT NOT NULL,
    subject TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT auth_identities_issuer_subject_unique UNIQUE (issuer, subject)
);

CREATE INDEX auth_identities_account_id_idx ON auth_identities (account_id);

CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    token_hash BYTEA NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    user_agent TEXT,
    created_ip TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT sessions_token_hash_unique UNIQUE (token_hash)
);

CREATE INDEX sessions_account_id_idx ON sessions (account_id);
CREATE INDEX sessions_expires_at_idx ON sessions (expires_at);
