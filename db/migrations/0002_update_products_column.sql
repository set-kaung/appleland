-- Add migration script here
ALTER TABLE products
RENAME COLUMN sku to barcode;