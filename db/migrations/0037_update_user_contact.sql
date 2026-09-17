-- Add migration script here
ALTER TABLE user_contact
    ADD user_id UUID NOT NULL REFERENCES users (id);

CREATE INDEX IF NOT EXISTS idx_delivery_contact_user_id ON user_contact (user_id);