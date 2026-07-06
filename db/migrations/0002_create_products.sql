-- Add migration script here
CREATE TABLE IF NOT EXISTS products(
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),

    name TEXT NOT NULL,
    barcode TEXT NOT NULL UNIQUE,
    description TEXT,
    category_id UUID NOT NULL,
    sub_category_id UUID,
    price numeric(19,2) NOT NULL,
    is_active BOOLEAN NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    created_by uuid NOT NULL,
    updated_by uuid NOT NULL

    FOREIGN KEY (category_id)
        REFERENCES categories(id),

    FOREIGN KEY (sub_category_id)
        REFERENCES categories(id)
)