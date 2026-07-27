-- Add migration script here
CREATE INDEX IF NOT EXISTS idx_sales_admin ON admin_sales (admin_id);