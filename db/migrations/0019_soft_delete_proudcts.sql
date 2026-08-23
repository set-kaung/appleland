-- Add migration script here
ALTER TABLE products
ADD is_deleted boolean NOT NULL DEFAULT false,
ADD deleted_at timestamptz,
ADD deleted_by uuid REFERENCES admins (id);