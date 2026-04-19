ALTER TABLE profiles ADD COLUMN years_of_experience INTEGER;
ALTER TABLE profiles ADD COLUMN salary_min INTEGER;
ALTER TABLE profiles ADD COLUMN salary_max INTEGER;
ALTER TABLE profiles ADD COLUMN salary_currency TEXT DEFAULT 'USD';
ALTER TABLE profiles ADD COLUMN languages JSONB DEFAULT '[]';
