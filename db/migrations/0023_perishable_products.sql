-- Add migration script here
ALTER TABLE inventory
ADD COLUMN is_perishable bool NOT NULL DEFAULT false;

ALTER TABLE inventory
ADD COLUMN expiry_date TIMESTAMPTZ;