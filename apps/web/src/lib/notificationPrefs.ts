export type NotificationPrefKey = 'new_jobs_found' | 'job_reactivated' | 'application_due_soon';

const STORAGE_KEY = (profileId: string) => `jc_notif_prefs_${profileId}`;

const DEFAULT_PREFS: Record<NotificationPrefKey, boolean> = {
  new_jobs_found: true,
  job_reactivated: true,
  application_due_soon: true,
};

export function readNotificationPrefs(profileId: string): Record<NotificationPrefKey, boolean> {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY(profileId));
    if (!raw) return { ...DEFAULT_PREFS };
    return { ...DEFAULT_PREFS, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULT_PREFS };
  }
}

export function writeNotificationPrefs(
  profileId: string,
  prefs: Record<NotificationPrefKey, boolean>,
) {
  window.localStorage.setItem(STORAGE_KEY(profileId), JSON.stringify(prefs));
}
