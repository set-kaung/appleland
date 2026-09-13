-- Add migration script here
ALTER TABLE playground_accounts
ADD COLUMN free_ticket_eligible INT NOT NULL DEFAULT 0;

ALTER TABLE playground_purchase_tokens DROP COLUMN free_quantity;

ALTER TABLE playground_purchase_tokens
RENAME COLUMN purchased_quantity TO total_quantity;