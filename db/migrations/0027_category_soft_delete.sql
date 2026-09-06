-- Add migration script here
ALTER TABLE categories
ADD COLUMN is_deleted boolean NOT NULL DEFAULT false,
ADD COLUMN deleted_at TIMESTAMPTZ,
ADD COLUMN deleted_by UUID;

ALTER TABLE categories 
ADD CONSTRAINT fk_deleted_by
FOREIGN KEY (deleted_by)
REFERENCES admins (id);

