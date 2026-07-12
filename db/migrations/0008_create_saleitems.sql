-- Add migration script here
CREATE TABLE IF NOT EXISTS sale_items(
    id uuid primary key unique not null default gen_random_uuid(),
    sale_id uuid not null,
    product_id uuid not null,
    quantity int not null check ( quantity >=0 ),
    selling_price numeric(19,2) not null check ( selling_price >=0 ),
    cost_price    numeric(19,2) NOT NULL CHECK (cost_price >= 0),
    created_at timestamptz not null default now()
);