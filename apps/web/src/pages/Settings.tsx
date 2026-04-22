import { useQuery } from '@tanstack/react-query';
import { Bell, Settings as SettingsIcon, SlidersHorizontal, UserRound } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

import { getNotifications } from '../api/notifications';
import { getProfile, getStoredProfileRawText } from '../api/profiles';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { readProfileId } from '../lib/profileSession';
import { queryKeys } from '../queryKeys';
import { buildProfileCompletionState } from '../features/profile/profileCompletion';

const NOTIFICATION_PREVIEW_LIMIT = 20;

export default function Settings() {
  const navigate = useNavigate();
  const profileId = readProfileId();
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
    queryFn: () => getNotifications(profileId ?? undefined, NOTIFICATION_PREVIEW_LIMIT),
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

  return (
    <Page>
      <PageHeader
        title="Settings"
        description="A small settings surface for the active profile. Stored preferences are still profile-scoped; dedicated notification preferences are not implemented yet."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Settings' }]}
      />

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1.2fr)_minmax(320px,0.8fr)]">
        <Card className="border-border bg-card">
          <CardHeader className="gap-3">
            <div className="flex items-center gap-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-primary/15 bg-primary/10 text-primary">
                <UserRound className="h-4 w-4" />
              </div>
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

        <div className="space-y-6">
          <Card className="border-border bg-card">
            <CardHeader className="gap-3">
              <div className="flex items-center gap-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-primary/15 bg-primary/10 text-primary">
                  <SlidersHorizontal className="h-4 w-4" />
                </div>
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
              <div className="h-2 rounded-full bg-white/[0.06]">
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
                <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-primary/15 bg-primary/10 text-primary">
                  <Bell className="h-4 w-4" />
                </div>
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
              <SettingRow label="Dedicated notification prefs" value="Not implemented yet" />
              <Button variant="outline" onClick={() => navigate('/notifications')}>
                Open notifications
              </Button>
            </CardContent>
          </Card>
        </div>
      </div>
    </Page>
  );
}

function SettingRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">{label}</p>
      <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">{value}</p>
    </div>
  );
}
