-- Add migration script here
ALTER TABLE products
ADD COLUMN original_price NUMERIC(19, 2);

ALTER TABLE products
ADD CONSTRAINT products_original_price_check
CHECK (original_price IS NULL OR original_price > selling_price);
