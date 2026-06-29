ALTER TABLE products
    ADD CONSTRAINT fk_products_created_by
    FOREIGN KEY (created_by)
    REFERENCES admins(id);

ALTER TABLE products
    ADD CONSTRAINT fk_products_updated_by
    FOREIGN KEY (updated_by)
    REFERENCES admins(id);