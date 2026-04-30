export type InternalAppNotificationType =
  | 'new_jobs_found'
  | 'job_reactivated'
  | 'market_company_hiring_again'
  | 'application_due_soon';

export type EngineNotification = {
  id: string;
  profile_id: string;
  type: InternalAppNotificationType;
  title: string;
  body?: string | null;
  payload?: Record<string, unknown> | null;
  read_at?: string | null;
  created_at: string;
};

export type EngineNotificationsResponse = {
  notifications: EngineNotification[];
};

export type EngineUnreadNotificationsCountResponse = {
  profile_id: string;
  unread_count: number;
};

export type EngineNotificationPreferences = {
  profile_id: string;
  new_jobs_matching_profile: boolean;
  application_status_reminders: boolean;
  weekly_digest: boolean;
  market_intelligence_updates: boolean;
  created_at: string;
  updated_at: string;
};

export type EngineUpdateNotificationPreferencesRequest = Partial<{
  new_jobs_matching_profile: boolean;
  application_status_reminders: boolean;
  weekly_digest: boolean;
  market_intelligence_updates: boolean;
}>;
