-- Add migration script here
CREATE TABLE IF NOT EXISTS products(
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    barcode TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    price numeric(19,2) NOT NULL,
    category TEXT NOT NULL,
    sub_category TEXT,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)