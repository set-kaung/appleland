-- Add migration script here
ALTER TABLE sales
    ADD COLUMN is_instore bool NOT NULL DEFAULT true;