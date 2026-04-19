import { json, readStoredProfileId, request } from './client';
import type {
  EngineNotification,
  EngineNotificationsResponse,
  EngineUnreadNotificationsCountResponse,
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

export async function getNotifications(
  profileId?: string,
  limit: number = 20,
): Promise<AppNotification[]> {
  const resolvedId = profileId ?? readStoredProfileId() ?? undefined;
  if (!resolvedId) {
    return [];
  }

  const response = await request<EngineNotificationsResponse>(
    `/api/v1/notifications?profile_id=${encodeURIComponent(
      resolvedId,
    )}&limit=${encodeURIComponent(String(limit))}`,
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

export async function getUnreadCount(profileId?: string): Promise<number> {
  const resolvedId = profileId ?? readStoredProfileId() ?? undefined;
  if (!resolvedId) {
    return 0;
  }

  const response = await request<EngineUnreadNotificationsCountResponse>(
    `/api/v1/notifications/unread-count?profile_id=${encodeURIComponent(resolvedId)}`,
  );

  return response.unread_count;
}
