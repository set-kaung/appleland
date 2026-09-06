-- Add migration script here
ALTER TABLE orders 
ADD delivery_status text NOT NULL DEFAULT 'pending';