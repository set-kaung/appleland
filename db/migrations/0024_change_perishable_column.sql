-- Add migration script here
ALTER TABLE inventory
DROP COLUMN is_perishable;

ALTER TABLE products
ADD COLUMN is_perishable bool NOT NULL DEFAULT false;