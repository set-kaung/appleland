-- Add migration script here
ALTER TABLE sales ADD COLUMN payment_order_id TEXT UNIQUE;