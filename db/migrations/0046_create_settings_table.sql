-- Add migration script here
CREATE TABLE public.settings (
    key text PRIMARY KEY,
    value jsonb NOT NULL,
    description text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    created_by uuid NOT NULL REFERENCES admins (id),
    updaetd_by uuid NOT NULL REFERENCES admins (id)
);