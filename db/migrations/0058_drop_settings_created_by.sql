ALTER TABLE settings DROP COLUMN created_by;

ALTER TABLE settings ALTER COLUMN updated_by DROP NOT NULL;