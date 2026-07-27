-- Add migration script here

CREATE TABLE IF NOT EXISTS admin_sales (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    sale_id uuid NOT NULL,
    admin_id uuid NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT admin_sale_sale_fk FOREIGN KEY (sale_id) REFERENCES sales (id),
    CONSTRAINT admin_sale_admin_fk FOREIGN KEY (admin_id) REFERENCES admins (id)
)