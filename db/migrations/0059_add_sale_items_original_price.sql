ALTER TABLE sale_items ADD COLUMN original_price NUMERIC(19, 2);

ALTER TABLE sale_items
ADD CONSTRAINT sale_items_original_price_check CHECK (
    original_price IS NULL
    OR original_price > selling_price
);