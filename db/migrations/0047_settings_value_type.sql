-- Fix typo in settings table already applied by 0046.
ALTER TABLE public.settings RENAME COLUMN updaetd_by TO updated_by;

-- Declare the Rust type the jsonb value holds, for type safety on new settings.
ALTER TABLE public.settings ADD COLUMN "type" text NOT NULL DEFAULT 'string';

ALTER TABLE public.settings
    ADD CONSTRAINT settings_value_type_check
    CHECK ("type" IN ('string', 'integer', 'decimal', 'boolean'));