-- Add migration script here
INSERT INTO settings (key, value, description, type)
VALUES
    ('kyat_per_point', '10000'::jsonb, 'how much kyat to point', 'decimal'),
    ('free_ticket_amount', '10'::jsonb, 'how much tickets get 1 free', 'integer'),
    ('one_point_to_kyat', '10000'::jsonb, 'how much one point to kyat is', 'decimal');
