CREATE TABLE IF NOT EXISTS user_store_points (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    user_id UUID UNIQUE NOT NULL REFERENCES users (id),
    balance INT NOT NULL DEFAULT 0 CHECK (balance >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_point_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    user_id UUID NOT NULL REFERENCES users (id),
    sale_id UUID REFERENCES sales (id),
    delta INT NOT NULL CHECK (delta != 0),
    reason TEXT NOT NULL CHECK (
        reason IN (
            'earn',
            'redeem',
            'adjust',
            'refund'
        )
    ),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_point_transactions_user_id ON user_point_transactions (user_id);

ALTER TABLE sales
ADD COLUMN points_earned INT NOT NULL DEFAULT 0 CHECK (points_earned >= 0);

ALTER TABLE sales
ADD COLUMN points_redeemed INT NOT NULL DEFAULT 0 CHECK (points_redeemed >= 0);

ALTER TABLE sales
ADD COLUMN points_discount_amount NUMERIC(20, 2) NOT NULL DEFAULT 0 CHECK (points_discount_amount >= 0);