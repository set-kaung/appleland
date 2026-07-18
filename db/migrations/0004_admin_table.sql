-- Add migration script here

CREATE TYPE admin_role AS ENUM ('sale_admin', 'playground_admin', 'super_admin');

create table admins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    role admin_role NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);