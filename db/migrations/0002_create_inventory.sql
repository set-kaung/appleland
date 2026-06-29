-- Add migration script here
CREATE TABLE inventory (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),

    product_id UUID NOT NULL,
    quantity integer NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    created_by uuid NOT NULL,
    updated_by uuid NOT NULL
);