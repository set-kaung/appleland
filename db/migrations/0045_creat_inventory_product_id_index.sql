-- Add migration script here
CREATE INDEX IF NOT EXISTS idx_product_id ON inventory (product_id);