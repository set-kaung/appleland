CREATE TYPE adjustment_reason AS ENUM ('damaged', 'expired', 'miscounted', 'corrected');

CREATE TABLE inventory_adjustments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    inventory_id UUID REFERENCES inventory(id),
    quantity_change INTEGER NOT NULL CHECK (quantity_change != 0),
    reason adjustment_reason NOT NULL,
    recorded_by UUID NOT NULL REFERENCES admins(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    note TEXT
);
