ALTER TABLE playground_purchases
ADD COLUMN token_id UUID REFERENCES playground_purchase_tokens (id);

-- Backfill existing purchase rows by matching the claiming user and the
-- token's claimed_at timestamp. This is best-effort; if a user claimed
-- multiple tokens within the same one-second window, the mapping may be
-- wrong. Any unmappable rows will cause the NOT NULL step below to fail.
UPDATE playground_purchases pp
SET
    token_id = (
        SELECT t.id
        FROM playground_purchase_tokens t
        WHERE
            t.claimed_by = pp.user_id
            AND pp.created_at BETWEEN t.claimed_at - INTERVAL '1 second' AND t.claimed_at  + INTERVAL '1 second'
        ORDER BY ABS(
                EXTRACT(
                    EPOCH
                    FROM (pp.created_at - t.claimed_at)
                )
            )
        LIMIT 1
    )
WHERE
    pp.token_id IS NULL
    AND EXISTS (
        SELECT 1
        FROM playground_purchase_tokens t
        WHERE
            t.claimed_by = pp.user_id
            AND pp.created_at BETWEEN t.claimed_at - INTERVAL '1 second' AND t.claimed_at  + INTERVAL '1 second'
    );

ALTER TABLE playground_purchases
ALTER COLUMN token_id
SET NOT NULL;