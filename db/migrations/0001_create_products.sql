-- Add migration script here
CREATE TABLE IF NOT EXISTS products(
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    sku text NOT NULL UNIQUE,
    name text NOT NULL,
    description text,
    price numeric(19,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)