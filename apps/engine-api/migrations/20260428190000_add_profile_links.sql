ALTER TABLE profiles
  ADD COLUMN portfolio_url TEXT,
  ADD COLUMN github_url TEXT,
  ADD COLUMN linkedin_url TEXT;

ALTER TABLE profiles
  ADD CONSTRAINT profiles_portfolio_url_http_check
  CHECK (portfolio_url IS NULL OR portfolio_url ~ '^https?://');

ALTER TABLE profiles
  ADD CONSTRAINT profiles_github_url_http_check
  CHECK (github_url IS NULL OR github_url ~ '^https?://');

ALTER TABLE profiles
  ADD CONSTRAINT profiles_linkedin_url_http_check
  CHECK (linkedin_url IS NULL OR linkedin_url ~ '^https?://');