-- Add migration script here
ALTER TABLE users
    ALTER COLUMN id SET DEFAULT gen_random_uuid();

ALTER TABLE users
    ADD COLUMN supabase_id UUID UNIQUE;

CREATE INDEX IF NOT EXISTS idx_users_supabase_id ON users (id);
