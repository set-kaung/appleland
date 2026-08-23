CREATE TABLE playground_accounts (
    user_id UUID PRIMARY KEY REFERENCES users (id),
    tickets_purchased INT NOT NULL DEFAULT 0,
    free_tickets_redeemed INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE playground_purchases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    user_id UUID NOT NULL REFERENCES users (id),
    quantity INT NOT NULL,
    is_free_redemption BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES admins (id)
);