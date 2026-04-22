ALTER TABLE profiles
ADD COLUMN IF NOT EXISTS search_preferences JSONB;
