-- Add migration script here
AlTER TABLE delivery_contact
    ALTER COLUMN phone_number SET NOT NULL;