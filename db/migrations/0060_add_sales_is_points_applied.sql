-- Add migration script here
ALTER TABLE sales
    ADD COLUMN is_points_applied bool NOT NULL DEFAULT false;
