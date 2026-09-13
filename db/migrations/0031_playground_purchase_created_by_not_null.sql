-- Add migration script here
ALTER TABLE playground_purchases
    ALTER COLUMN created_by SET NOT NULL;