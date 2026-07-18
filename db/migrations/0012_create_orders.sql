-- Add migration script here

CREATE TABLE IF NOT EXISTS orders
(
    id                  UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    sale_id             UUID UNIQUE NOT NULL,
    shipping_address_id UUID        NOT NULL,
    order_method        TEXT                 DEFAULT 'app order',
    created_at          timestamptz NOT NULL DEFAULT now(),
    created_by          UUID        NOT NULL,
    updated_at          timestamptz NOT NULL DEFAULT now(),
    updated_by          UUID        NOT NULL,
    CONSTRAINT shipping_address_fk FOREIGN KEY (shipping_address_id) REFERENCES shipping_addresses (id)
);