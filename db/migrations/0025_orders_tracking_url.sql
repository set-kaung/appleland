-- Add migration script here
ALTER TABLE orders
ADD COLUMN order_tracking_url TEXT;