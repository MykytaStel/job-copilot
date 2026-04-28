ALTER TABLE profiles
  ADD COLUMN experience JSONB DEFAULT '[]'::jsonb;
