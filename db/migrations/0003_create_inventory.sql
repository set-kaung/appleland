-- Add migration script here
CREATE TABLE inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    product_id UUID NOT NULL,
    quantity_received integer NOT NULL,
    quantity_remaining integer NOT NULL,
    unit_cost NUMERIC(19, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by uuid NOT NULL,
    updated_by uuid NOT NULL,
    CONSTRAINT check_remaining CHECK (quantity_remaining >= 0)
);