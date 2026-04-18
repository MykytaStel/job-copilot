import { useMemo, useState, type ReactNode } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  Ban,
  Bookmark,
  Building2,
  EyeOff,
  Plus,
  Search,
  ShieldCheck,
  ShieldOff,
  Star,
  ThumbsDown,
  Undo2,
  type LucideIcon,
} from 'lucide-react';
import toast from 'react-hot-toast';
import type { JobPosting } from '@job-copilot/shared';

import {
  addCompanyBlacklist,
  addCompanyWhitelist,
  getFeedback,
  getJobsFeed,
  removeCompanyBlacklist,
  removeCompanyWhitelist,
  unhideJob,
  unmarkJobBadFit,
  unsaveJob,
} from '../api';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card, CardContent, CardHeader } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { StatCard } from '../components/ui/StatCard';
import { PageHeader } from '../components/ui/SectionHeader';
import { cn } from '../lib/cn';
import { queryKeys } from '../queryKeys';

function readProfileId() {
  return window.localStorage.getItem('engine_api_profile_id');
}

type FeedbackTab = 'saved' | 'hidden' | 'bad-fit' | 'companies';
type FeedbackListTone = Exclude<FeedbackTab, 'companies'>;

const FEEDBACK_TAB_META: Array<{
  id: FeedbackTab;
  label: string;
  description: string;
  icon: LucideIcon;
}> = [
  {
    id: 'saved',
    label: 'Saved',
    description: 'High-intent roles you want to revisit and potentially act on.',
    icon: Bookmark,
  },
  {
    id: 'hidden',
    label: 'Hidden',
    description: 'Suppressed roles that should stay out of the main ranking feed.',
    icon: EyeOff,
  },
  {
    id: 'bad-fit',
    label: 'Bad Fit',
    description: 'Explicit mismatches used as negative ranking evidence.',
    icon: ThumbsDown,
  },
  {
    id: 'companies',
    label: 'Companies',
    description: 'Allow and block lists that steer future ranking toward preferred employers.',
    icon: Building2,
  },
];

const JOB_ROW_TONE_STYLES: Record<
  FeedbackListTone,
  {
    badge: 'default' | 'warning' | 'danger';
    badgeLabel: string;
    iconClass: string;
    actionClass: string;
  }
> = {
  saved: {
    badge: 'default',
    badgeLabel: 'Positive signal',
    iconClass: 'border-primary/20 bg-primary/10 text-primary',
    actionClass: 'text-primary hover:text-primary',
  },
  hidden: {
    badge: 'warning',
    badgeLabel: 'Suppressed',
    iconClass: 'border-border bg-white/[0.04] text-muted-foreground',
    actionClass: 'text-muted-foreground hover:text-foreground',
  },
  'bad-fit': {
    badge: 'danger',
    badgeLabel: 'Negative signal',
    iconClass: 'border-destructive/20 bg-destructive/10 text-destructive',
    actionClass: 'text-destructive hover:text-destructive',
  },
};

function JobRow({
  job,
  tone,
  actionLabel,
  onAction,
  isPending,
}: {
  job: JobPosting;
  tone: FeedbackListTone;
  actionLabel: string;
  onAction: (jobId: string) => void;
  isPending: boolean;
}) {
  const presentation = job.presentation;
  const sourceLabel = presentation?.sourceLabel ?? job.primaryVariant?.source ?? 'source';
  const toneStyle = JOB_ROW_TONE_STYLES[tone];
  const metaItems = [
    presentation?.locationLabel,
    presentation?.workModeLabel,
    presentation?.salaryLabel,
    presentation?.freshnessLabel,
  ].filter(Boolean) as string[];

  return (
    <Card className="overflow-hidden border-border bg-card">
      <CardContent className="flex flex-col gap-4 px-5 py-5 md:flex-row md:items-start md:justify-between">
        <div className="flex min-w-0 gap-4">
          <div
            className={cn(
              'mt-0.5 flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border',
              toneStyle.iconClass,
            )}
          >
            <Building2 className="h-4 w-4" />
          </div>
          <div className="min-w-0 space-y-3">
            <div className="flex flex-wrap items-center gap-2">
              <p className="m-0 truncate text-sm font-semibold text-card-foreground md:text-base">
                {presentation?.title ?? job.title}
              </p>
              <Badge
                variant={toneStyle.badge}
                className="px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
              >
                {toneStyle.badgeLabel}
              </Badge>
              <Badge
                variant="muted"
                className="px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
              >
                {sourceLabel}
              </Badge>
            </div>
            <p className="m-0 text-sm text-muted-foreground">
              {presentation?.company ?? job.company}
            </p>
            {presentation?.summary && (
              <p className="m-0 max-w-3xl text-sm leading-6 text-muted-foreground">
                {presentation.summary}
              </p>
            )}
            {metaItems.length > 0 && (
              <div className="flex flex-wrap gap-2">
                {metaItems.map((item) => (
                  <span
                    key={item}
                    className="inline-flex items-center rounded-full border border-border bg-white/[0.04] px-2.5 py-1 text-[11px] text-muted-foreground"
                  >
                    {item}
                  </span>
                ))}
              </div>
            )}
          </div>
        </div>
        <div className="flex shrink-0 items-center md:pl-4">
          <Button
            variant="ghost"
            size="sm"
            className={cn('h-10 rounded-xl px-3', toneStyle.actionClass)}
            onClick={() => onAction(job.id)}
            disabled={isPending}
          >
            <Undo2 size={13} />
            {actionLabel}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

function Section({
  title,
  icon,
  description,
  count,
  children,
}: {
  title: string;
  icon: ReactNode;
  description?: string;
  count: number;
  children: ReactNode;
}) {
  return (
    <Card className="border-border bg-card">
      <CardHeader className="gap-3">
        <div className="flex items-center gap-3">
          <span className="flex h-10 w-10 items-center justify-center rounded-xl border border-border bg-white/[0.04] text-content-muted">
            {icon}
          </span>
          <div>
            <h2 className="m-0 text-[15px] font-semibold text-content">{title}</h2>
            {description ? (
              <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">{description}</p>
            ) : null}
          </div>
          <Badge variant="muted" className="ml-auto rounded-lg px-2 py-0.5 text-xs">
            {count}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">{children}</CardContent>
    </Card>
  );
}

function CompanyPanel({
  title,
  description,
  count,
  value,
  placeholder,
  accent,
  onChange,
  onSubmit,
  isSubmitting,
  emptyMessage,
  children,
}: {
  title: string;
  description: string;
  count: number;
  value: string;
  placeholder: string;
  accent: 'success' | 'danger';
  onChange: (value: string) => void;
  onSubmit: () => void;
  isSubmitting: boolean;
  emptyMessage: string;
  children: ReactNode;
}) {
  return (
    <Card className="border-border bg-card">
      <CardHeader className="gap-3">
        <div className="flex items-center gap-2">
          <h2 className="m-0 text-base font-semibold text-card-foreground">{title}</h2>
          <Badge
            variant={accent}
            className="ml-auto px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
          >
            {count}
          </Badge>
        </div>
        <p className="m-0 text-sm leading-6 text-muted-foreground">{description}</p>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="rounded-2xl border border-border/70 bg-white/[0.03] p-3">
          <div className="flex gap-2">
            <input
              type="text"
              value={value}
              onChange={(event) => onChange(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') onSubmit();
              }}
              placeholder={placeholder}
              className="h-11 flex-1 rounded-xl border border-border bg-background/70 px-3"
            />
            <Button
              type="button"
              variant="outline"
              size="icon"
              className="h-11 w-11 rounded-xl"
              onClick={onSubmit}
              disabled={isSubmitting || !value.trim()}
            >
              <Plus className="h-4 w-4" />
            </Button>
          </div>
        </div>
        {count === 0 ? (
          <EmptyState message={emptyMessage} className="px-4 py-5 text-left" />
        ) : (
          <div className="flex flex-col gap-3">{children}</div>
        )}
      </CardContent>
    </Card>
  );
}

function CompanyRow({
  companyName,
  accent,
  badgeLabel,
  description,
  moveTitle,
  onMove,
  onRemove,
  isMovePending,
  isRemovePending,
}: {
  companyName: string;
  accent: 'success' | 'danger';
  badgeLabel: string;
  description: string;
  moveTitle: string;
  onMove: () => void;
  onRemove: () => void;
  isMovePending: boolean;
  isRemovePending: boolean;
}) {
  const iconClass =
    accent === 'success'
      ? 'border-fit-excellent/20 bg-fit-excellent/10 text-fit-excellent'
      : 'border-destructive/20 bg-destructive/10 text-destructive';
  const rowClass =
    accent === 'success'
      ? 'border-fit-excellent/20 bg-fit-excellent/5'
      : 'border-destructive/20 bg-destructive/5';

  return (
    <Card className={cn('border', rowClass)}>
      <CardContent className="flex items-start justify-between gap-4 px-4 py-4">
        <div className="flex min-w-0 items-start gap-3">
          <div className={cn('flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border', iconClass)}>
            <Building2 className="h-4 w-4" />
          </div>
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <p className="m-0 text-sm font-semibold text-foreground">{companyName}</p>
              <Badge
                variant={accent}
                className="px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
              >
                {badgeLabel}
              </Badge>
            </div>
            <p className="mt-1 mb-0 text-xs leading-6 text-muted-foreground">{description}</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-muted-foreground"
            onClick={onMove}
            disabled={isMovePending}
            title={moveTitle}
          >
            {accent === 'success' ? <Ban className="h-4 w-4" /> : <Star className="h-4 w-4" />}
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={onRemove}
            disabled={isRemovePending}
          >
            Remove
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

export default function FeedbackCenter() {
  const profileId = readProfileId();
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = useState<FeedbackTab>('saved');
  const [searchQuery, setSearchQuery] = useState('');
  const [whitelistInput, setWhitelistInput] = useState('');
  const [blacklistInput, setBlacklistInput] = useState('');

  const { data: jobsFeed, isLoading: jobsLoading } = useQuery({
    queryKey: queryKeys.jobs.filtered('all', null, profileId),
    queryFn: () => getJobsFeed({ limit: 200 }),
    enabled: !!profileId,
  });

  const { data: feedbackOverview, isLoading: feedbackLoading } = useQuery({
    queryKey: queryKeys.feedback.profile(profileId ?? ''),
    queryFn: () => getFeedback(profileId!),
    enabled: !!profileId,
  });

  const allJobs = jobsFeed?.jobs ?? [];
  const savedJobs = allJobs.filter((job) => job.feedback?.saved);
  const hiddenJobs = allJobs.filter((job) => job.feedback?.hidden);
  const badFitJobs = allJobs.filter((job) => job.feedback?.badFit);
  const whitelistedCompanies = (feedbackOverview?.companies ?? []).filter(
    (company) => company.status === 'whitelist',
  );
  const blacklistedCompanies = (feedbackOverview?.companies ?? []).filter(
    (company) => company.status === 'blacklist',
  );
  const summary = feedbackOverview?.summary;
  const normalizedSearch = searchQuery.trim().toLowerCase();

  const filterJobsBySearch = (jobs: JobPosting[]) =>
    jobs.filter((job) => {
      if (!normalizedSearch) return true;

      const title = job.presentation?.title ?? job.title;
      const company = job.presentation?.company ?? job.company;

      return (
        title.toLowerCase().includes(normalizedSearch) ||
        company.toLowerCase().includes(normalizedSearch)
      );
    });

  const filteredSavedJobs = useMemo(
    () => filterJobsBySearch(savedJobs),
    [normalizedSearch, savedJobs],
  );
  const filteredHiddenJobs = useMemo(
    () => filterJobsBySearch(hiddenJobs),
    [normalizedSearch, hiddenJobs],
  );
  const filteredBadFitJobs = useMemo(
    () => filterJobsBySearch(badFitJobs),
    [normalizedSearch, badFitJobs],
  );

  const tabCounts: Record<FeedbackTab, number> = {
    saved: savedJobs.length,
    hidden: hiddenJobs.length,
    'bad-fit': badFitJobs.length,
    companies: whitelistedCompanies.length + blacklistedCompanies.length,
  };

  const activeTabMeta = FEEDBACK_TAB_META.find((tab) => tab.id === activeTab) ?? FEEDBACK_TAB_META[0];

  function invalidateAfterAction() {
    void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
    if (profileId) {
      void queryClient.invalidateQueries({ queryKey: queryKeys.feedback.profile(profileId) });
    }
  }

  const unsaveMutation = useMutation({
    mutationFn: (jobId: string) => unsaveJob(profileId!, jobId),
    onSuccess: () => {
      invalidateAfterAction();
      toast.success('Збережено скасовано');
    },
    onError: (error: unknown) =>
      toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const unhideMutation = useMutation({
    mutationFn: (jobId: string) => unhideJob(profileId!, jobId),
    onSuccess: () => {
      invalidateAfterAction();
      toast.success('Вакансію показано знову');
    },
    onError: (error: unknown) =>
      toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const unmarkBadFitMutation = useMutation({
    mutationFn: (jobId: string) => unmarkJobBadFit(profileId!, jobId),
    onSuccess: () => {
      invalidateAfterAction();
      toast.success('Позначку bad fit знято');
    },
    onError: (error: unknown) =>
      toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const removeWhitelistMutation = useMutation({
    mutationFn: (companyName: string) => removeCompanyWhitelist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterAction();
      toast.success('Видалено зі списку дозволених');
    },
    onError: (error: unknown) =>
      toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const removeBlacklistMutation = useMutation({
    mutationFn: (companyName: string) => removeCompanyBlacklist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterAction();
      toast.success('Видалено зі списку заблокованих');
    },
    onError: (error: unknown) =>
      toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const addWhitelistMutation = useMutation({
    mutationFn: (companyName: string) => addCompanyWhitelist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterAction();
      setWhitelistInput('');
      toast.success('Компанію додано до whitelist');
    },
    onError: (error: unknown) =>
      toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const addBlacklistMutation = useMutation({
    mutationFn: (companyName: string) => addCompanyBlacklist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterAction();
      setBlacklistInput('');
      toast.success('Компанію додано до blacklist');
    },
    onError: (error: unknown) =>
      toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const moveCompanyMutation = useMutation({
    mutationFn: ({
      companyName,
      nextStatus,
    }: {
      companyName: string;
      nextStatus: 'whitelist' | 'blacklist';
    }) =>
      nextStatus === 'whitelist'
        ? addCompanyWhitelist(profileId!, companyName)
        : addCompanyBlacklist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterAction();
      toast.success('Статус компанії оновлено');
    },
    onError: (error: unknown) =>
      toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  function submitCompany(status: 'whitelist' | 'blacklist') {
    const value = (status === 'whitelist' ? whitelistInput : blacklistInput).trim();
    if (!value) return;

    if (status === 'whitelist') {
      addWhitelistMutation.mutate(value);
      return;
    }

    addBlacklistMutation.mutate(value);
  }

  if (!profileId) {
    return (
      <Page>
        <EmptyState message="Create a profile to view feedback." />
      </Page>
    );
  }

  const isLoading = jobsLoading || feedbackLoading;

  return (
    <Page>
      <PageHeader
        title="Feedback Center"
        description="Manage saved jobs, hidden roles, bad fits, and company preferences."
        breadcrumb={[
          { label: 'Dashboard', href: '/' },
          { label: 'Feedback' },
        ]}
      />

      {isLoading ? (
        <EmptyState message="Loading feedback…" />
      ) : (
        <>
          <Card className="overflow-hidden border-border bg-card">
            <CardContent className="p-0">
              <div className="relative">
                <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-primary/8 via-accent/5 to-transparent" />
                <div className="relative flex flex-col gap-6 p-7 lg:flex-row lg:items-end lg:justify-between">
                  <div className="max-w-3xl space-y-3">
                    <div className="flex flex-wrap gap-2">
                      <Badge
                        variant="default"
                        className="border-0 bg-primary/15 px-2 py-0.5 text-xs text-primary"
                      >
                        Feedback loops
                      </Badge>
                      <Badge
                        variant="muted"
                        className="px-2 py-0.5 text-[10px] uppercase tracking-wide"
                      >
                        Saved, hidden, bad-fit, companies
                      </Badge>
                    </div>
                    <h2 className="m-0 text-2xl font-bold text-card-foreground lg:text-3xl">
                      Train the ranking engine with explicit feedback
                    </h2>
                    <p className="m-0 text-sm leading-7 text-muted-foreground lg:text-base">
                      Saved jobs, hidden roles, bad fits, and company allow/block lists directly
                      shape what rises or disappears from the feed.
                    </p>
                  </div>
                  <div className="grid gap-3 sm:grid-cols-3 lg:min-w-[420px]">
                    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
                      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                        Saved
                      </p>
                      <p className="m-0 mt-2 text-2xl font-bold text-card-foreground">
                        {savedJobs.length}
                      </p>
                    </div>
                    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
                      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                        Bad fit
                      </p>
                      <p className="m-0 mt-2 text-2xl font-bold text-card-foreground">
                        {badFitJobs.length}
                      </p>
                    </div>
                    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
                      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                        Company lists
                      </p>
                      <p className="m-0 mt-2 text-2xl font-bold text-card-foreground">
                        {tabCounts.companies}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          {summary && (
            <div className="grid grid-cols-2 gap-4 lg:grid-cols-5">
              <StatCard title="Saved" value={summary.savedJobsCount} icon={Bookmark} />
              <StatCard title="Hidden" value={summary.hiddenJobsCount} icon={EyeOff} />
              <StatCard title="Bad Fit" value={summary.badFitJobsCount} icon={ThumbsDown} />
              <StatCard
                title="Whitelisted"
                value={summary.whitelistedCompaniesCount}
                icon={ShieldCheck}
              />
              <StatCard
                title="Blacklisted"
                value={summary.blacklistedCompaniesCount}
                icon={ShieldOff}
              />
            </div>
          )}

          <Card className="border-border bg-card">
            <CardContent className="space-y-6 px-6 py-6">
              <div className="grid gap-5 lg:grid-cols-[minmax(0,1fr)_320px] lg:items-start">
                <div className="space-y-4">
                  <div className="flex w-full flex-wrap gap-2">
                    {FEEDBACK_TAB_META.map((tab) => (
                      <button
                        key={tab.id}
                        type="button"
                        onClick={() => setActiveTab(tab.id)}
                        className={cn(
                          'inline-flex items-center gap-2 rounded-full border px-3 py-2 text-sm transition-colors',
                          activeTab === tab.id
                            ? 'border-primary bg-primary text-primary-foreground'
                            : 'border-border bg-surface-elevated/50 text-muted-foreground hover:bg-surface-hover hover:text-foreground',
                        )}
                      >
                        <tab.icon className="h-4 w-4" />
                        <span>{tab.label}</span>
                        <span
                          className={cn(
                            'rounded-full px-1.5 py-0.5 text-[11px] leading-none',
                            activeTab === tab.id
                              ? 'bg-white/20 text-white'
                              : 'bg-black/20 text-muted-foreground',
                          )}
                        >
                          {tabCounts[tab.id]}
                        </span>
                      </button>
                    ))}
                  </div>
                  <p className="m-0 max-w-3xl text-sm leading-6 text-muted-foreground">
                    {activeTabMeta.description}
                  </p>
                </div>

                <div className="space-y-3 rounded-2xl border border-border/70 bg-white/[0.03] p-4">
                  <p className="m-0 text-[11px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
                    Active slice
                  </p>
                  <div className="flex items-center gap-3">
                    <div className="flex h-11 w-11 items-center justify-center rounded-2xl border border-primary/15 bg-primary/10 text-primary">
                      <activeTabMeta.icon className="h-4 w-4" />
                    </div>
                    <div>
                      <p className="m-0 text-sm font-semibold text-card-foreground">
                        {activeTabMeta.label}
                      </p>
                      <p className="m-0 mt-1 text-xs text-muted-foreground">
                        {activeTab === 'companies'
                          ? `${tabCounts.companies} companies tracked`
                          : `${searchQuery ? 'Filtered' : 'All'} jobs in this list`}
                      </p>
                    </div>
                  </div>
                  {activeTab !== 'companies' && (
                    <div className="relative">
                      <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                      <input
                        type="search"
                        value={searchQuery}
                        onChange={(event) => setSearchQuery(event.target.value)}
                        placeholder="Search jobs..."
                        className="h-11 w-full rounded-xl border border-border bg-background/70 pl-9"
                      />
                    </div>
                  )}
                </div>
              </div>

              {activeTab === 'saved' && (
                <Section
                  title="Saved Jobs"
                  icon={<Bookmark size={16} />}
                  description="High-confidence jobs you kept for follow-up, tailoring, or application."
                  count={filteredSavedJobs.length}
                >
                  {filteredSavedJobs.length === 0 ? (
                    <EmptyState
                      message={searchQuery ? 'No saved jobs match this query.' : 'No saved jobs.'}
                      className="px-4 py-5 text-left"
                    />
                  ) : (
                    <div className="flex flex-col gap-3">
                      {filteredSavedJobs.map((job) => (
                        <JobRow
                          key={job.id}
                          job={job}
                          tone="saved"
                          actionLabel="Unsave"
                          onAction={(jobId) => unsaveMutation.mutate(jobId)}
                          isPending={unsaveMutation.isPending}
                        />
                      ))}
                    </div>
                  )}
                </Section>
              )}

              {activeTab === 'hidden' && (
                <Section
                  title="Hidden Jobs"
                  icon={<EyeOff size={16} />}
                  description="Suppressed jobs stay out of the main feed until you restore them."
                  count={filteredHiddenJobs.length}
                >
                  {filteredHiddenJobs.length === 0 ? (
                    <EmptyState
                      message={searchQuery ? 'No hidden jobs match this query.' : 'No hidden jobs.'}
                      className="px-4 py-5 text-left"
                    />
                  ) : (
                    <div className="flex flex-col gap-3">
                      {filteredHiddenJobs.map((job) => (
                        <JobRow
                          key={job.id}
                          job={job}
                          tone="hidden"
                          actionLabel="Unhide"
                          onAction={(jobId) => unhideMutation.mutate(jobId)}
                          isPending={unhideMutation.isPending}
                        />
                      ))}
                    </div>
                  )}
                </Section>
              )}

              {activeTab === 'bad-fit' && (
                <Section
                  title="Bad Fit"
                  icon={<ThumbsDown size={16} />}
                  description="Negative examples influence future ranking and reduce similar recommendations."
                  count={filteredBadFitJobs.length}
                >
                  {filteredBadFitJobs.length === 0 ? (
                    <EmptyState
                      message={
                        searchQuery
                          ? 'No bad-fit jobs match this query.'
                          : 'No jobs marked as bad fit.'
                      }
                      className="px-4 py-5 text-left"
                    />
                  ) : (
                    <div className="flex flex-col gap-3">
                      {filteredBadFitJobs.map((job) => (
                        <JobRow
                          key={job.id}
                          job={job}
                          tone="bad-fit"
                          actionLabel="Unmark"
                          onAction={(jobId) => unmarkBadFitMutation.mutate(jobId)}
                          isPending={unmarkBadFitMutation.isPending}
                        />
                      ))}
                    </div>
                  )}
                </Section>
              )}

              {activeTab === 'companies' && (
                <div className="grid gap-6 lg:grid-cols-2">
                  <CompanyPanel
                    title="Whitelisted Companies"
                    description="Jobs from these companies should be prioritized in the feed."
                    count={whitelistedCompanies.length}
                    value={whitelistInput}
                    placeholder="Add company to priority list..."
                    accent="success"
                    onChange={setWhitelistInput}
                    onSubmit={() => submitCompany('whitelist')}
                    isSubmitting={addWhitelistMutation.isPending}
                    emptyMessage="No whitelisted companies."
                  >
                    {whitelistedCompanies.map((company) => (
                      <CompanyRow
                        key={company.normalizedCompanyName}
                        companyName={company.companyName}
                        accent="success"
                        badgeLabel="Priority"
                        description="Prioritized for future ranking and shortlist views."
                        moveTitle="Move to blacklist"
                        onMove={() =>
                          moveCompanyMutation.mutate({
                            companyName: company.companyName,
                            nextStatus: 'blacklist',
                          })
                        }
                        onRemove={() => removeWhitelistMutation.mutate(company.companyName)}
                        isMovePending={moveCompanyMutation.isPending}
                        isRemovePending={removeWhitelistMutation.isPending}
                      />
                    ))}
                  </CompanyPanel>

                  <CompanyPanel
                    title="Blacklisted Companies"
                    description="Jobs from these companies should be hidden from the main feed."
                    count={blacklistedCompanies.length}
                    value={blacklistInput}
                    placeholder="Add company to block list..."
                    accent="danger"
                    onChange={setBlacklistInput}
                    onSubmit={() => submitCompany('blacklist')}
                    isSubmitting={addBlacklistMutation.isPending}
                    emptyMessage="No blacklisted companies."
                  >
                    {blacklistedCompanies.map((company) => (
                      <CompanyRow
                        key={company.normalizedCompanyName}
                        companyName={company.companyName}
                        accent="danger"
                        badgeLabel="Blocked"
                        description="Suppressed from ranking and hidden in future feeds."
                        moveTitle="Move to whitelist"
                        onMove={() =>
                          moveCompanyMutation.mutate({
                            companyName: company.companyName,
                            nextStatus: 'whitelist',
                          })
                        }
                        onRemove={() => removeBlacklistMutation.mutate(company.companyName)}
                        isMovePending={moveCompanyMutation.isPending}
                        isRemovePending={removeBlacklistMutation.isPending}
                      />
                    ))}
                  </CompanyPanel>
                </div>
              )}
            </CardContent>
          </Card>
        </>
      )}
    </Page>
  );
}
