import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';

import {
  getNotificationPreferences,
  getNotifications,
  patchNotificationPreferences,
  type NotificationPreferences,
} from '../../api/notifications';
import { Button } from '../../components/ui/Button';
import { SurfaceMetric } from '../../components/ui/Surface';
import { cn } from '../../lib/cn';
import { readProfileId } from '../../lib/profileSession';
import { queryKeys } from '../../queryKeys';
import { SettingsSection } from './settingsShared';

const NOTIFICATION_PREVIEW_LIMIT = 20;

type NotificationPreferenceKey =
  | 'newJobsMatchingProfile'
  | 'applicationStatusReminders'
  | 'weeklyDigest'
  | 'marketIntelligenceUpdates';

const NOTIFICATION_PREF_LABELS: {
  key: NotificationPreferenceKey;
  title: string;
  description: string;
}[] = [
  {
    key: 'newJobsMatchingProfile',
    title: 'New jobs matching search profile',
    description: 'Notify me when new roles match my saved search profile.',
  },
  {
    key: 'applicationStatusReminders',
    title: 'Application status change reminders',
    description: 'Remind me when an application needs a follow-up.',
  },
  {
    key: 'weeklyDigest',
    title: 'Weekly digest',
    description: 'Send a weekly summary of new jobs, feedback, and progress.',
  },
  {
    key: 'marketIntelligenceUpdates',
    title: 'Market intelligence updates',
    description: 'Notify me when salary or market trends change.',
  },
];

export function NotificationsSettingsSection() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const profileId = readProfileId();

  const { data: notificationPreferences, isLoading: notificationPreferencesLoading } = useQuery({
    queryKey: queryKeys.notifications.preferences(profileId ?? 'none'),
    queryFn: getNotificationPreferences,
    enabled: !!profileId,
  });
  const { data: notifications = [] } = useQuery({
    queryKey: queryKeys.notifications.list(profileId ?? 'none', NOTIFICATION_PREVIEW_LIMIT),
    queryFn: () => getNotifications(NOTIFICATION_PREVIEW_LIMIT),
    enabled: !!profileId,
  });

  const notificationPreferencesMutation = useMutation({
    mutationFn: patchNotificationPreferences,
    onMutate: async (patch) => {
      if (!profileId) return;

      const queryKey = queryKeys.notifications.preferences(profileId);
      await queryClient.cancelQueries({ queryKey });

      const previous = queryClient.getQueryData<NotificationPreferences>(queryKey);
      if (previous) {
        queryClient.setQueryData<NotificationPreferences>(queryKey, { ...previous, ...patch });
      }

      return { previous };
    },
    onError: (_error, _patch, context) => {
      if (!profileId || !context?.previous) return;
      queryClient.setQueryData(queryKeys.notifications.preferences(profileId), context.previous);
    },
    onSettled: () => {
      if (!profileId) return;
      void queryClient.invalidateQueries({
        queryKey: queryKeys.notifications.preferences(profileId),
      });
    },
  });

  const unreadCount = notifications.filter((n) => !n.readAt).length;

  function toggleNotificationPreference(
    key: NotificationPreferenceKey,
    preferences: NotificationPreferences,
  ) {
    notificationPreferencesMutation.mutate({ [key]: !preferences[key] });
  }

  return (
    <SettingsSection title="Notifications">
      <div className="space-y-4">
        <SurfaceMetric>
          <p className="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
            Unread
          </p>
          <p className="mt-2 text-2xl font-semibold text-foreground">{unreadCount}</p>
        </SurfaceMetric>

        <div className="rounded-[var(--radius-lg)] border border-border bg-surface-soft/40 p-4">
          <div className="mb-4">
            <p className="text-sm font-semibold text-foreground">Notification preferences</p>
            <p className="mt-1 text-sm text-muted-foreground">
              These preferences are saved in engine-api and scoped to the active profile.
            </p>
          </div>

          {notificationPreferencesLoading && (
            <p className="text-sm text-muted-foreground">Loading notification preferences…</p>
          )}

          {!notificationPreferencesLoading && notificationPreferences && (
            <div className="space-y-3">
              {NOTIFICATION_PREF_LABELS.map(({ key, title, description }) => {
                const enabled = notificationPreferences[key];

                return (
                  <div
                    key={key}
                    className="flex items-start justify-between gap-4 rounded-[var(--radius-md)] border border-border bg-surface p-3"
                  >
                    <div>
                      <p className="text-sm font-medium text-foreground">{title}</p>
                      <p className="mt-1 text-xs text-muted-foreground">{description}</p>
                    </div>

                    <button
                      type="button"
                      aria-pressed={enabled}
                      onClick={() => toggleNotificationPreference(key, notificationPreferences)}
                      disabled={notificationPreferencesMutation.isPending}
                      className={cn(
                        'relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus:outline-none disabled:cursor-not-allowed disabled:opacity-60',
                        enabled ? 'bg-primary' : 'bg-surface-soft',
                      )}
                    >
                      <span
                        className={cn(
                          'inline-block h-4 w-4 rounded-full bg-white shadow transition-transform',
                          enabled ? 'translate-x-4' : 'translate-x-0',
                        )}
                      />
                    </button>
                  </div>
                );
              })}
            </div>
          )}

          {notificationPreferencesMutation.isError && (
            <p className="mt-3 text-sm text-danger">
              Could not save notification preferences. Please try again.
            </p>
          )}
        </div>

        <Button variant="outline" onClick={() => navigate('/notifications')}>
          Open notifications
        </Button>
      </div>
    </SettingsSection>
  );
}
