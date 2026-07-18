-- Add migration script here

CREATE TABLE IF NOT EXISTS shipping_addresses
(
    id           UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    apartment_no TEXT        NOT NULL,
    road         TEXT        NOT NULL,
    city         TEXT        NOT NULL,
    postal_code  TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
)