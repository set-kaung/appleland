-- Add migration script here
ALTER TABLE shipping_addresses
    DROP COLUMN IF EXISTS city;

ALTER TABLE shipping_addresses
    DROP COLUMN IF EXISTS apartment_no;

ALTER TABLE shipping_addresses
    DROP COLUMN IF EXISTS road;


ALTER TABLE shipping_addresses
    DROP COLUMN IF EXISTS postal_code;


ALTER TABLE shipping_addresses
    ADD COLUMN address_line_1 TEXT NOT NULL;

ALTER TABLE shipping_addresses
    ADD COLUMN phone_number TEXT;

ALTER TABLE shipping_addresses
    RENAME TO delivery_contact;