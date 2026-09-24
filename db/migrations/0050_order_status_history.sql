-- Add migration script here

-- Append-only log of delivery status changes per order.
CREATE TABLE order_status_history (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id   UUID NOT NULL REFERENCES orders (id) ON DELETE CASCADE,
    status     TEXT NOT NULL,
    changed_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_order_status_history_order_created
    ON order_status_history (order_id, created_at);