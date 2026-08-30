-- Add migration script here
ALTER TABLE inventory
ADD COLUMN received_at TIMESTAMPTZ NOT NULL DEFAULT NOW();