-- Add migration script here
ALTER TABLE products DROP CONSTRAINT IF EXISTS products_barcode_key;
DROP INDEX IF EXISTS products_barcode_key;

CREATE UNIQUE INDEX idx_products_barcode_active ON products (barcode) WHERE is_deleted IS FALSE;