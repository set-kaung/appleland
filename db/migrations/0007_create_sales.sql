-- Add migration script here
CREATE TABLE IF NOT EXISTS sales (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    total_amount numeric(20, 2) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT NOW()
);