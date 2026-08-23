-- Add migration script here
CREATE TABLE users (
    id UUID PRIMARY KEY, -- matches Supabase auth.users.id (JWT sub)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);