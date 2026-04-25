import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  Bell,
  Route,
  Settings as SettingsIcon,
  SlidersHorizontal,
  Target,
  UserRound,
} from 'lucide-react';
import { useNavigate } from 'react-router-dom';

import { getNotifications } from '../api/notifications';
import { getProfile, getStoredProfileRawText } from '../api/profiles';
import { AccentIconFrame } from '../components/ui/AccentIconFrame';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { SurfaceMetric } from '../components/ui/Surface';
import { readProfileId } from '../lib/profileSession';
import {
  readNotificationPrefs,
  writeNotificationPrefs,
  type NotificationPrefKey,
} from '../lib/notificationPrefs';
import { queryKeys } from '../queryKeys';
import { buildProfileCompletionState } from '../features/profile/profileCompletion';

const NOTIFICATION_PREVIEW_LIMIT = 20;

const NOTIF_LABELS: Record<NotificationPrefKey, string> = {
  new_jobs_found: 'New jobs found',
  job_reactivated: 'Job reactivated',
  application_due_soon: 'Application due soon',
};

export default function Settings() {
  const navigate = useNavigate();
  const profileId = readProfileId();
  const [notifPrefs, setNotifPrefs] = useState(() =>
    profileId ? readNotificationPrefs(profileId) : null,
  );

  function toggleNotifPref(key: NotificationPrefKey) {
    if (!profileId || !notifPrefs) return;
    const updated = { ...notifPrefs, [key]: !notifPrefs[key] };
    setNotifPrefs(updated);
    writeNotificationPrefs(profileId, updated);
  }
  const { data: profile } = useQuery({
    queryKey: queryKeys.profile.root(),
    queryFn: getProfile,
  });
  const { data: rawText = '' } = useQuery({
    queryKey: queryKeys.profile.rawText(),
    queryFn: getStoredProfileRawText,
    enabled: !!profile,
  });
  const { data: notifications = [] } = useQuery({
    queryKey: queryKeys.notifications.list(profileId ?? 'none', NOTIFICATION_PREVIEW_LIMIT),
    queryFn: () => getNotifications(NOTIFICATION_PREVIEW_LIMIT),
    enabled: !!profileId,
  });

  if (!profile) {
    return (
      <Page>
        <PageHeader
          title="Settings"
          description="Profile-scoped defaults and notification routing for the current operator context."
          breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Settings' }]}
        />
        <EmptyState
          icon={<SettingsIcon className="h-5 w-5" />}
          message="Settings need an active profile"
          description="Create or load a profile first so Job Copilot can scope preferences to the right candidate context."
        />
      </Page>
    );
  }

  const completion = buildProfileCompletionState({
    name: profile.name,
    email: profile.email,
    location: profile.location ?? '',
    rawText,
    yearsOfExperience: profile.yearsOfExperience ? String(profile.yearsOfExperience) : '',
    salaryMin: profile.salaryMin ? String(profile.salaryMin) : '',
    salaryMax: profile.salaryMax ? String(profile.salaryMax) : '',
    salaryCurrency: profile.salaryCurrency ?? '',
    languages: profile.languages,
    analysisReady: Boolean(profile.summary || profile.skills.length),
  });
  const unreadCount = notifications.filter((notification) => !notification.readAt).length;
  const persistedSearchPreferences = profile.searchPreferences;
  const persistedPreferenceCount =
    (persistedSearchPreferences?.targetRegions.length ?? 0) +
    (persistedSearchPreferences?.workModes.length ?? 0) +
    (persistedSearchPreferences?.preferredRoles.length ?? 0) +
    (persistedSearchPreferences?.allowedSources.length ?? 0) +
    (persistedSearchPreferences?.includeKeywords.length ?? 0) +
    (persistedSearchPreferences?.excludeKeywords.length ?? 0);

  return (
    <Page>
      <PageHeader
        title="Settings"
        description="A small settings surface for the active profile. Canonical profile editing and search-profile building stay in Profile & Search; Settings summarizes what is already live."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Settings' }]}
      />

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1.2fr)_minmax(320px,0.8fr)]">
        <div className="space-y-6">
          <Card className="border-border bg-card">
            <CardHeader className="gap-3">
              <div className="flex items-center gap-3">
                <AccentIconFrame size="md">
                  <UserRound className="h-4 w-4" />
                </AccentIconFrame>
                <div>
                  <CardTitle>Profile defaults</CardTitle>
                  <p className="m-0 text-sm leading-6 text-muted-foreground">
                    The persisted profile still drives ranking, AI context, and notifications scope.
                  </p>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-3 md:grid-cols-2">
                <SettingRow label="Name" value={profile.name} />
                <SettingRow label="Email" value={profile.email} />
                <SettingRow label="Location" value={profile.location ?? 'Not set'} />
                <SettingRow
                  label="Compensation"
                  value={
                    profile.salaryMin && profile.salaryMax
                      ? `${profile.salaryMin}-${profile.salaryMax} ${profile.salaryCurrency}`
                      : 'Not set'
                  }
                />
              </div>
              <div className="flex flex-wrap gap-2">
                {profile.languages.length > 0 ? (
                  profile.languages.map((language) => (
                    <Badge key={language} variant="muted" className="px-2 py-0.5">
                      {language}
                    </Badge>
                  ))
                ) : (
                  <Badge variant="muted" className="px-2 py-0.5">
                    No languages set
                  </Badge>
                )}
              </div>
              <Button onClick={() => navigate('/profile')}>Open profile</Button>
            </CardContent>
          </Card>

          <Card className="border-border bg-card">
            <CardHeader className="gap-3">
              <div className="flex items-center gap-3">
                <AccentIconFrame size="md">
                  <Target className="h-4 w-4" />
                </AccentIconFrame>
                <div>
                  <CardTitle>Search profile scope</CardTitle>
                  <p className="m-0 text-sm leading-6 text-muted-foreground">
                    Search filters persist on the profile record, but building and running search
                    still happens in Profile &amp; Search.
                  </p>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-3 md:grid-cols-2">
                <SettingRow
                  label="Saved search inputs"
                  value={
                    persistedPreferenceCount > 0
                      ? `${persistedPreferenceCount} active filters`
                      : 'No saved filters yet'
                  }
                />
                <SettingRow
                  label="Persistence"
                  value={persistedSearchPreferences ? 'Profile-scoped' : 'Local until saved'}
                />
                <SettingRow
                  label="Target regions"
                  value={String(persistedSearchPreferences?.targetRegions.length ?? 0)}
                />
                <SettingRow
                  label="Preferred roles"
                  value={String(persistedSearchPreferences?.preferredRoles.length ?? 0)}
                />
              </div>
              <p className="m-0 text-sm leading-6 text-muted-foreground">
                This page stays summary-only for now so the search workflow does not split across
                multiple routes.
              </p>
              <Button variant="outline" onClick={() => navigate('/profile')}>
                Open Profile &amp; Search
              </Button>
            </CardContent>
          </Card>
        </div>

        <div className="space-y-6">
          <Card className="border-border bg-card">
            <CardHeader className="gap-3">
              <div className="flex items-center gap-3">
                <AccentIconFrame size="md">
                  <SlidersHorizontal className="h-4 w-4" />
                </AccentIconFrame>
                <div>
                  <CardTitle>Profile readiness</CardTitle>
                  <p className="m-0 text-sm leading-6 text-muted-foreground">
                    Completion reflects whether the stored profile is rich enough for ranking and AI
                    workflows.
                  </p>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-3">
              <p className="m-0 text-3xl font-semibold text-card-foreground">
                {completion.percent}%
              </p>
              <div className="h-2 rounded-full bg-surface-soft">
                <div
                  className="h-2 rounded-full bg-[image:var(--gradient-button)]"
                  style={{ width: `${completion.percent}%` }}
                />
              </div>
              <p className="m-0 text-sm text-muted-foreground">
                {completion.missingLabels.length > 0
                  ? `Still missing: ${completion.missingLabels.join(', ')}`
                  : 'All current checkpoints are covered.'}
              </p>
            </CardContent>
          </Card>

          <Card className="border-border bg-card">
            <CardHeader className="gap-3">
              <div className="flex items-center gap-3">
                <AccentIconFrame size="md">
                  <Bell className="h-4 w-4" />
                </AccentIconFrame>
                <div>
                  <CardTitle>Notification scope</CardTitle>
                  <p className="m-0 text-sm leading-6 text-muted-foreground">
                    Notifications are currently profile-scoped and generated from ingestion-driven
                    changes.
                  </p>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-3">
              <SettingRow label="Unread inbox items" value={String(unreadCount)} />
              {notifPrefs && (
                <div className="space-y-2 pt-1">
                  <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                    Show in dashboard
                  </p>
                  {(Object.keys(NOTIF_LABELS) as NotificationPrefKey[]).map((key) => (
                    <label
                      key={key}
                      className="flex cursor-pointer items-center justify-between gap-3 rounded-[var(--radius-md)] border border-border/40 bg-surface-muted px-3 py-2"
                    >
                      <span className="text-sm text-foreground">{NOTIF_LABELS[key]}</span>
                      <button
                        role="switch"
                        aria-checked={notifPrefs[key]}
                        onClick={() => toggleNotifPref(key)}
                        className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus:outline-none ${
                          notifPrefs[key] ? 'bg-primary' : 'bg-surface-soft'
                        }`}
                      >
                        <span
                          className={`pointer-events-none inline-block h-3.5 w-3.5 transform rounded-full bg-white shadow transition-transform ${
                            notifPrefs[key] ? 'translate-x-4' : 'translate-x-0.5'
                          }`}
                        />
                      </button>
                    </label>
                  ))}
                  <p className="text-[11px] text-muted-foreground/70">
                    Stored locally. Backend notification routing coming soon.
                  </p>
                </div>
              )}
              <Button variant="outline" onClick={() => navigate('/notifications')}>
                Open notifications
              </Button>
            </CardContent>
          </Card>

          <Card className="border-border bg-card">
            <CardHeader className="gap-3">
              <div className="flex items-center gap-3">
                <AccentIconFrame size="md">
                  <Route className="h-4 w-4" />
                </AccentIconFrame>
                <div>
                  <CardTitle>Route plan</CardTitle>
                  <p className="m-0 text-sm leading-6 text-muted-foreground">
                    Keep navigation minimal by using each route for a single, already-implemented
                    responsibility.
                  </p>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-3">
              <SettingRow
                label="`/profile`"
                value="Edit profile, filters, build search, run ranking"
              />
              <SettingRow label="`/notifications`" value="Review the profile-scoped inbox" />
              <SettingRow
                label="`/settings`"
                value="Summarize active scope and future prefs surface"
              />
            </CardContent>
          </Card>
        </div>
      </div>
    </Page>
  );
}

function SettingRow({ label, value }: { label: string; value: string }) {
  return (
    <SurfaceMetric>
      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">{label}</p>
      <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">{value}</p>
    </SurfaceMetric>
  );
}
