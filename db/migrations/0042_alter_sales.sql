-- Add migration script here
ALTER TABLE sales
ADD COLUMN payment_method TEXT,
ADD COLUMN payment_provider TEXT;

ALTER TABLE sales ALTER COLUMN is_instore DROP DEFAULT;