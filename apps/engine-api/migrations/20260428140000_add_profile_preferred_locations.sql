ALTER TABLE profiles
ADD COLUMN IF NOT EXISTS preferred_locations JSONB DEFAULT '[]'::jsonb;

UPDATE profiles
SET preferred_locations = '[]'::jsonb
WHERE preferred_locations IS NULL;

ALTER TABLE profiles
ADD CONSTRAINT profiles_preferred_locations_array
CHECK (preferred_locations IS NULL OR jsonb_typeof(preferred_locations) = 'array');
