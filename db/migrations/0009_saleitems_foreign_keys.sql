-- Add migration script here
ALTER TABLE sale_items
ADD CONSTRAINT fk_sale_product
FOREIGN KEY (product_id)
REFERENCES products(id);


ALTER TABLE sale_items
ADD CONSTRAINT fk_sale_items_sale
FOREIGN KEY (sale_id)
REFERENCES sales(id);