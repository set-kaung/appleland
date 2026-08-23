-- Add migration script here
ALTER TABLE playground_purchases
ADD unit_price numeric(10, 2) NOT NULL;