-- Add migration script here
CREATE TABLE user_contact
(
    id             UUID PRIMARY KEY NOT NULL DEFAULT pg_catalog.gen_random_uuid(),
    address_line_1 text             NOT NULL,
    phone_number   text             NOT NULL,
    created_at     timestamptz      NOT NULL DEFAULT NOW(),
    updated_at     timestamptz      NOT NULL DEFAULT NOW()
);

