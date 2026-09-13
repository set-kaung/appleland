-- Add migration script here

CREATE TABLE playground_coupons (
    id uuid primary key DEFAULT gen_random_uuid (),
    user_id uuid NOT NULL,
    is_used boolean NOT NULL DEFAULT FALSE,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_playground_coupon_user FOREIGN KEY (user_id) REFERENCES users (id)
);