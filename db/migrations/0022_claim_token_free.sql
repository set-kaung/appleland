-- Add migration script here
ALTER TABLE playground_purchase_tokens
    RENAME COLUMN quantity TO purchased_quantity;

ALTER TABLE playground_purchase_tokens
    ADD COLUMN free_quantity INT NULL;