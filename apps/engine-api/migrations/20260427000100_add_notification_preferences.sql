CREATE TABLE IF NOT EXISTS notification_preferences (
  profile_id TEXT PRIMARY KEY REFERENCES profiles(id) ON DELETE CASCADE,
  new_jobs_matching_profile BOOLEAN NOT NULL DEFAULT TRUE,
  application_status_reminders BOOLEAN NOT NULL DEFAULT TRUE,
  weekly_digest BOOLEAN NOT NULL DEFAULT TRUE,
  market_intelligence_updates BOOLEAN NOT NULL DEFAULT TRUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS notification_preferences_profile_id_idx
  ON notification_preferences(profile_id);