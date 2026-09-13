-- Add migration script here
ALTER TABLE playground_purchase_tokens
ADD COLUMN unit_price numeric(10,2) NOT NULL;