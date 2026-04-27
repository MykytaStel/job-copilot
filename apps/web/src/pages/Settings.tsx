import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  Bell,
  Settings as SettingsIcon,
  ShieldCheck,
  SlidersHorizontal,
  Target,
  UserRound,
} from 'lucide-react';
import { useNavigate } from 'react-router-dom';

import {
  getNotificationPreferences,
  getNotifications,
  patchNotificationPreferences,
  type AppNotificationType,
  type NotificationPreferences,
} from '../api/notifications';
import { getProfile, getStoredProfileRawText } from '../api/profiles';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { SurfaceMetric } from '../components/ui/Surface';
import { cn } from '../lib/cn';
import { readProfileId } from '../lib/profileSession';
import {
  readDensity,
  writeDensity,
  readSortMode,
  writeSortMode,
  type DensityMode,
  type SortMode,
} from '../lib/displayPrefs';
import { queryKeys } from '../queryKeys';
import { buildProfileCompletionState } from '../features/profile/profileCompletion';

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

const NOTIF_LABELS: Record<AppNotificationType, string> = {
  new_jobs_found: 'New jobs found',
  job_reactivated: 'Job reactivated',
  application_due_soon: 'Application due soon',
};

type SectionId = 'profile' | 'search' | 'notifications' | 'display' | 'privacy';

const SECTIONS: {
  id: SectionId;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
}[] = [
  { id: 'profile', label: 'Profile & Account', icon: UserRound },
  { id: 'search', label: 'Search Preferences', icon: Target },
  { id: 'notifications', label: 'Notifications', icon: Bell },
  { id: 'display', label: 'Display', icon: SlidersHorizontal },
  { id: 'privacy', label: 'Data & Privacy', icon: ShieldCheck },
];

export default function Settings() {
  const navigate = useNavigate();
	const queryClient = useQueryClient();
  const profileId = readProfileId();
  const [activeSection, setActiveSection] = useState<SectionId>('profile');
  const [density, setDensityState] = useState<DensityMode>(() => readDensity());
  const [sortPref, setSortPrefState] = useState<SortMode>(() => readSortMode());
	const {
		data: notificationPreferences,
		isLoading: notificationPreferencesLoading,
	} = useQuery({
		queryKey: queryKeys.notifications.preferences(profileId ?? 'none'),
		queryFn: getNotificationPreferences,
		enabled: !!profileId,
	});
	const notificationPreferencesMutation = useMutation({
		mutationFn: patchNotificationPreferences,
		onMutate: async (patch) => {
			if (!profileId) return;

			const queryKey = queryKeys.notifications.preferences(profileId);
			await queryClient.cancelQueries({ queryKey });

			const previous =
				queryClient.getQueryData<NotificationPreferences>(queryKey);

			if (previous) {
				queryClient.setQueryData<NotificationPreferences>(queryKey, {
					...previous,
					...patch,
				});
			}

			return { previous };
		},
		onError: (_error, _patch, context) => {
			if (!profileId || !context?.previous) return;

			queryClient.setQueryData(
				queryKeys.notifications.preferences(profileId),
				context.previous,
			);
		},
		onSettled: () => {
			if (!profileId) return;

			void queryClient.invalidateQueries({
				queryKey: queryKeys.notifications.preferences(profileId),
			});
		},
	});

  function setDensity(value: DensityMode) {
    writeDensity(value);
    setDensityState(value);
  }

  function setSortPref(value: SortMode) {
    writeSortMode(value);
    setSortPrefState(value);
  }

  function toggleNotificationPreference(
		key: NotificationPreferenceKey,
		preferences: NotificationPreferences,
	) {
		notificationPreferencesMutation.mutate({
			[key]: !preferences[key],
		});
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

  const unreadCount = notifications.filter((n) => !n.readAt).length;
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
        description="Profile-scoped preferences and operator defaults."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Settings' }]}
      />

      <div className="flex flex-col gap-6 sm:flex-row sm:gap-0">
        {/* Sidebar nav — horizontal scroll on mobile, vertical column on sm+ */}
        <nav className="flex shrink-0 flex-row gap-1 overflow-x-auto pb-1 sm:w-52 sm:flex-col sm:overflow-x-visible sm:border-r sm:border-border sm:pb-0 sm:pr-4">
          {SECTIONS.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => setActiveSection(id)}
              className={cn(
                'flex items-center gap-2.5 whitespace-nowrap rounded-[var(--radius-md)] px-3 py-2 text-sm font-medium transition-colors',
                activeSection === id
                  ? 'bg-surface-muted text-foreground'
                  : 'text-muted-foreground hover:bg-surface-muted/60 hover:text-foreground',
              )}
            >
              <Icon className="h-4 w-4 shrink-0" />
              {label}
            </button>
          ))}
        </nav>

        {/* Content panel */}
        <div className="min-w-0 flex-1 sm:pl-8">
          {activeSection === 'profile' && (
            <SettingsSection title="Profile & Account">
              <div className="space-y-6">
                <div className="grid gap-3 md:grid-cols-2">
                  <SettingRow label="Name" value={profile.name} />
                  <SettingRow label="Email" value={profile.email} />
                  <SettingRow label="Location" value={profile.location ?? 'Not set'} />
                  <SettingRow
                    label="Compensation"
                    value={
                      profile.salaryMin && profile.salaryMax
                        ? `${profile.salaryMin}–${profile.salaryMax} ${profile.salaryCurrency}`
                        : 'Not set'
                    }
                  />
                </div>

                <div>
                  <p className="mb-2 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                    Languages
                  </p>
                  <div className="flex flex-wrap gap-2">
                    {profile.languages.length > 0 ? (
                      profile.languages.map((lang) => (
                        <Badge key={lang} variant="muted" className="px-2 py-0.5">
                          {lang}
                        </Badge>
                      ))
                    ) : (
                      <Badge variant="muted" className="px-2 py-0.5">
                        No languages set
                      </Badge>
                    )}
                  </div>
                </div>

                <div>
                  <p className="mb-1 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                    Profile readiness
                  </p>
                  <p className="mt-1 text-2xl font-semibold text-card-foreground">
                    {completion.percent}%
                  </p>
                  <div className="mt-2 h-2 rounded-full bg-surface-soft">
                    <div
                      className="h-2 rounded-full bg-[image:var(--gradient-button)]"
                      style={{ width: `${completion.percent}%` }}
                    />
                  </div>
                  {completion.missingLabels.length > 0 && (
                    <p className="mt-1 text-sm text-muted-foreground">
                      Missing: {completion.missingLabels.join(', ')}
                    </p>
                  )}
                </div>

                <Button onClick={() => navigate('/profile')}>Open profile</Button>
              </div>
            </SettingsSection>
          )}

          {activeSection === 'search' && (
            <SettingsSection title="Search Preferences">
              <div className="space-y-4">
                <div className="grid gap-3 md:grid-cols-2">
                  <SettingRow
                    label="Active filters"
                    value={
                      persistedPreferenceCount > 0
                        ? `${persistedPreferenceCount} active filters`
                        : 'None set'
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
                <p className="text-sm text-muted-foreground">
                  Full search preference editing lives in Profile &amp; Search.
                </p>
                <Button variant="outline" onClick={() => navigate('/profile')}>
                  Open Profile &amp; Search
                </Button>
              </div>
            </SettingsSection>
          )}

          {activeSection === 'notifications' && (
						<SettingsSection title="Notifications">
							<div className="space-y-4">
								<SurfaceMetric>
										<p className="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
											Unread
										</p>
										<p className="mt-2 text-2xl font-semibold text-foreground">
											{unreadCount}
										</p>
								</SurfaceMetric>

								<div className="rounded-[var(--radius-lg)] border border-border bg-surface-soft/40 p-4">
									<div className="mb-4">
										<p className="text-sm font-semibold text-foreground">
											Notification preferences
										</p>
										<p className="mt-1 text-sm text-muted-foreground">
											These preferences are saved in engine-api and scoped to the active profile.
										</p>
									</div>

									{notificationPreferencesLoading && (
										<p className="text-sm text-muted-foreground">
											Loading notification preferences…
										</p>
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
															<p className="text-sm font-medium text-foreground">
																{title}
															</p>
															<p className="mt-1 text-xs text-muted-foreground">
																{description}
															</p>
														</div>

														<button
															type="button"
															aria-pressed={enabled}
															onClick={() =>
																toggleNotificationPreference(key, notificationPreferences)
															}
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
					)}

          {activeSection === 'display' && (
            <SettingsSection title="Display">
              <div className="space-y-6">
                <div className="space-y-2">
                  <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                    Job card density
                  </p>
                  <div className="flex flex-wrap gap-2">
                    {(['compact', 'normal', 'comfortable'] as DensityMode[]).map((value) => (
                      <button
                        key={value}
                        onClick={() => setDensity(value)}
                        className={cn(
                          'rounded-[var(--radius-md)] border px-3 py-1.5 text-sm font-medium capitalize transition-colors',
                          density === value
                            ? 'border-primary bg-primary/10 text-primary'
                            : 'border-border text-muted-foreground hover:border-border/80 hover:text-foreground',
                        )}
                      >
                        {value}
                      </button>
                    ))}
                  </div>
                  <p className="text-[11px] text-muted-foreground/70">
                    Controls spacing between job cards in the feed.
                  </p>
                </div>

                <div className="space-y-2">
                  <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                    Default job sort
                  </p>
                  <div className="flex flex-wrap gap-2">
                    {([
                      { value: 'relevance', label: 'Relevance' },
                      { value: 'date', label: 'Date' },
                      { value: 'salary', label: 'Salary' },
                    ] as { value: SortMode; label: string }[]).map(({ value, label }) => (
                      <button
                        key={value}
                        onClick={() => setSortPref(value)}
                        className={cn(
                          'rounded-[var(--radius-md)] border px-3 py-1.5 text-sm font-medium transition-colors',
                          sortPref === value
                            ? 'border-primary bg-primary/10 text-primary'
                            : 'border-border text-muted-foreground hover:border-border/80 hover:text-foreground',
                        )}
                      >
                        {label}
                      </button>
                    ))}
                  </div>
                  <p className="text-[11px] text-muted-foreground/70">
                    Applied when you open the dashboard. Relevance requires an active profile.
                  </p>
                </div>
              </div>
            </SettingsSection>
          )}

          {activeSection === 'privacy' && (
            <SettingsSection title="Data & Privacy">
              <p className="text-sm text-muted-foreground">
                Data retention, export, and privacy controls will be available in a future release.
              </p>
            </SettingsSection>
          )}
        </div>
      </div>
    </Page>
  );
}

function SettingsSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="space-y-6">
      <div className="border-b border-border pb-4">
        <h2 className="text-base font-semibold text-foreground">{title}</h2>
      </div>
      {children}
    </div>
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
