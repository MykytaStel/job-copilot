import type { AppNotification } from '../../api/notifications';

export function countUnreadNotifications(notifications: AppNotification[]) {
  return notifications.filter((notification) => !notification.readAt).length;
}

export function resolveNotificationsViewState({
  profileId,
  isLoading,
  error,
  notifications,
}: {
  profileId: string | null;
  isLoading: boolean;
  error: unknown;
  notifications: AppNotification[];
}) {
  if (!profileId) {
    return 'missing_profile' as const;
  }
  if (isLoading) {
    return 'loading' as const;
  }
  if (error) {
    return 'error' as const;
  }
  if (notifications.length === 0) {
    return 'empty' as const;
  }
  return 'ready' as const;
}
