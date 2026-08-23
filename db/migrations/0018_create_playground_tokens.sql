-- Add migration script here

CREATE TYPE playground_token_status AS ENUM ( 'pending', 'claimed' ,'expired');

CREATE TABLE playground_purchase_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    quantity INT NOT NULL CHECK (quantity > 0),
    status playground_token_status NOT NULL DEFAULT 'pending', -- pending | claimed | expired
    created_by UUID NOT NULL REFERENCES admins (id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    claimed_by UUID REFERENCES users (id),
    claimed_at TIMESTAMPTZ
);

CREATE INDEX idx_tokens_status_expires ON playground_purchase_tokens (status, expires_at);