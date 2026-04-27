import { json, request } from './client';
import { hasToken } from '../lib/authSession';
import type {
  EngineNotification,
  EngineNotificationsResponse,
  EngineUnreadNotificationsCountResponse,
	EngineNotificationPreferences,
  EngineUpdateNotificationPreferencesRequest,
} from './engine-types';


export type AppNotificationType = 'new_jobs_found' | 'job_reactivated' | 'application_due_soon';

export type AppNotification = {
  id: string;
  profileId: string;
  type: AppNotificationType;
  title: string;
  body?: string;
  payload?: Record<string, unknown>;
  readAt?: string;
  createdAt: string;
};

export type NotificationPreferences = {
  profileId: string;
  newJobsMatchingProfile: boolean;
  applicationStatusReminders: boolean;
  weeklyDigest: boolean;
  marketIntelligenceUpdates: boolean;
  createdAt: string;
  updatedAt: string;
};

export type UpdateNotificationPreferencesInput = Partial<{
  newJobsMatchingProfile: boolean;
  applicationStatusReminders: boolean;
  weeklyDigest: boolean;
  marketIntelligenceUpdates: boolean;
}>;

function mapNotification(notification: EngineNotification): AppNotification {
  return {
    id: notification.id,
    profileId: notification.profile_id,
    type: notification.type,
    title: notification.title,
    body: notification.body ?? undefined,
    payload: notification.payload ?? undefined,
    readAt: notification.read_at ?? undefined,
    createdAt: notification.created_at,
  };
}

function mapNotificationPreferences(
  preferences: EngineNotificationPreferences,
): NotificationPreferences {
  return {
    profileId: preferences.profile_id,
    newJobsMatchingProfile: preferences.new_jobs_matching_profile,
    applicationStatusReminders: preferences.application_status_reminders,
    weeklyDigest: preferences.weekly_digest,
    marketIntelligenceUpdates: preferences.market_intelligence_updates,
    createdAt: preferences.created_at,
    updatedAt: preferences.updated_at,
  };
}

function toEngineNotificationPreferencesPatch(
  input: UpdateNotificationPreferencesInput,
): EngineUpdateNotificationPreferencesRequest {
  return {
    new_jobs_matching_profile: input.newJobsMatchingProfile,
    application_status_reminders: input.applicationStatusReminders,
    weekly_digest: input.weeklyDigest,
    market_intelligence_updates: input.marketIntelligenceUpdates,
  };
}

export async function getNotifications(limit: number = 20): Promise<AppNotification[]> {
  if (!hasToken()) return [];

  const response = await request<EngineNotificationsResponse>(
    `/api/v1/notifications?limit=${encodeURIComponent(String(limit))}`,
  );

  return response.notifications.map(mapNotification);
}

export async function markNotificationRead(id: string): Promise<AppNotification> {
  const notification = await request<EngineNotification>(
    `/api/v1/notifications/${encodeURIComponent(id)}/read`,
    json('POST', {}),
  );

  return mapNotification(notification);
}

export async function getUnreadCount(): Promise<number> {
  if (!hasToken()) return 0;

  const response = await request<EngineUnreadNotificationsCountResponse>(
    `/api/v1/notifications/unread-count`,
  );

  return response.unread_count;
}

export async function getNotificationPreferences(): Promise<NotificationPreferences> {
  const response = await request<EngineNotificationPreferences>(
    '/api/v1/notifications/preferences',
  );

  return mapNotificationPreferences(response);
}

export async function patchNotificationPreferences(
  input: UpdateNotificationPreferencesInput,
): Promise<NotificationPreferences> {
  const response = await request<EngineNotificationPreferences>(
    '/api/v1/notifications/preferences',
    json('PATCH', toEngineNotificationPreferencesPatch(input)),
  );

  return mapNotificationPreferences(response);
}