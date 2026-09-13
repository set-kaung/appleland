-- Add migration script here
ALTER TABLE playground_coupons
    ADD COLUMN updated_at timestamptz NOT NULL DEFAULT NOW();