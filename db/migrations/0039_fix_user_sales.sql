-- Add migration script here
ALTER TABLE user_sale RENAME TO user_sales;

ALTER TABLE user_sales
ADD COLUMN sale_id UUID NOT NULL,
ADD CONSTRAINT fk_user_sales_sale_id FOREIGN KEY (sale_id) REFERENCES sales (id);