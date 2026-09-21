-- Replay protection for payment webhooks: one row per consumed nonce.
CREATE TABLE payment_webhook_nonces (
    nonce       TEXT PRIMARY KEY,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_payment_webhook_nonces_consumed_at
    ON payment_webhook_nonces (consumed_at);