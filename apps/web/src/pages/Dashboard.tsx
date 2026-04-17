import type { ReactElement } from 'react';
import { useEffect, useMemo, useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import {
  ArrowRight,
  Bookmark,
  Briefcase,
  CalendarDays,
  KanbanSquare,
  Search,
  Send,
  SortAsc,
  Sparkles,
  TrendingUp,
  XCircle,
  Zap,
} from 'lucide-react';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card, CardContent, CardHeader } from '../components/ui/Card';
import { AIInsightPanel } from '../components/ui/AIInsightPanel';
import { FilterChips } from '../components/ui/FilterChips';
import { EmptyState } from '../components/ui/EmptyState';
import { JobCard, JobCardSkeleton } from '../components/ui/JobCard';
import { Page } from '../components/ui/Page';
import { SectionHeader } from '../components/ui/SectionHeader';
import { StatCard } from '../components/ui/StatCard';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type {
  Application,
  ApplicationStatus,
  JobFeedSummary,
  JobPosting,
} from '@job-copilot/shared';

import {
  type RankedJob,
  addCompanyBlacklist,
  addCompanyWhitelist,
  createApplication,
  getApplications,
  getDashboardStats,
  getJobsFeed,
  hideJobForProfile,
  markJobBadFit,
  markJobSaved,
  removeCompanyBlacklist,
  removeCompanyWhitelist,
  getSources,
  rerankJobs,
  unmarkJobBadFit,
} from '../api';
import { logJobImpressionsOnce } from '../features/events/jobImpressions';
import { queryKeys } from '../queryKeys';

function readProfileId() {
  return window.localStorage.getItem('engine_api_profile_id');
}


const STATUS_COLUMNS: ApplicationStatus[] = [
  'saved',
  'applied',
  'interview',
  'offer',
  'rejected',
];

const STATUS_ICONS = {
  saved: <Bookmark size={14} />,
  applied: <Send size={14} />,
  interview: <CalendarDays size={14} />,
  offer: <Briefcase size={14} />,
  rejected: <XCircle size={14} />,
} satisfies Record<ApplicationStatus, ReactElement>;

type LifecycleFilter = 'all' | 'active' | 'inactive' | 'reactivated';

const LIFECYCLE_TABS: { value: LifecycleFilter; label: string }[] = [
  { value: 'all', label: 'Всі' },
  { value: 'active', label: 'Активні' },
  { value: 'inactive', label: 'Зникли' },
  { value: 'reactivated', label: 'Повернулись' },
];

const DEFAULT_LIFECYCLE_FILTER: LifecycleFilter = 'all';

function readLifecycleFilter(searchParams: URLSearchParams): LifecycleFilter {
  const lifecycle = searchParams.get('lifecycle');

  if (
    lifecycle === 'active' ||
    lifecycle === 'inactive' ||
    lifecycle === 'reactivated'
  ) {
    return lifecycle;
  }

  return DEFAULT_LIFECYCLE_FILTER;
}

function readSourceFilter(searchParams: URLSearchParams): string | null {
  const source = searchParams.get('source')?.trim();
  return source ? source : null;
}

export default function Dashboard() {
  const queryClient = useQueryClient();
  const [searchParams, setSearchParams] = useSearchParams();
  const [search, setSearch] = useState('');
  const [sortByScore, setSortByScore] = useState(false);
  const lifecycleFilter = readLifecycleFilter(searchParams);
  const sourceFilter = readSourceFilter(searchParams);
  const profileId = readProfileId();

  const updateFilters = ({
    lifecycle,
    source,
  }: {
    lifecycle?: LifecycleFilter;
    source?: string | null;
  }) => {
    const nextSearchParams = new URLSearchParams(searchParams);

    if (lifecycle !== undefined) {
      if (lifecycle === DEFAULT_LIFECYCLE_FILTER) {
        nextSearchParams.delete('lifecycle');
      } else {
        nextSearchParams.set('lifecycle', lifecycle);
      }
    }

    if (source !== undefined) {
      if (source) {
        nextSearchParams.set('source', source);
      } else {
        nextSearchParams.delete('source');
      }
    }

    setSearchParams(nextSearchParams, { replace: true });
  };

  const { data: jobsFeed, error: jobsError, isLoading: jobsLoading } = useQuery<{
    jobs: JobPosting[];
    summary: JobFeedSummary;
  }>({
    queryKey: queryKeys.jobs.filtered(lifecycleFilter, sourceFilter, profileId),
    queryFn: () =>
      getJobsFeed({
        lifecycle: lifecycleFilter === 'all' ? undefined : lifecycleFilter,
        source: sourceFilter ?? undefined,
        limit: 200,
      }),
  });

  const {
    data: sources = [],
    error: sourcesError,
  } = useQuery({
    queryKey: queryKeys.sources.all(),
    queryFn: getSources,
    staleTime: 5 * 60_000,
  });

  const allJobs = jobsFeed?.jobs ?? [];
  const jobSummary = jobsFeed?.summary;

  // ML rerank — fires once jobs are loaded and profile exists.
  const { data: rankData } = useQuery<RankedJob[]>({
    queryKey: queryKeys.ml.rerank(profileId ?? ''),
    queryFn: () => rerankJobs(profileId!, allJobs.map((j) => j.id)),
    enabled: !!profileId && allJobs.length > 0,
    staleTime: 5 * 60_000,
    retry: false,
  });

  const scoreById = useMemo(() => {
    const map = new Map<string, number>();
    rankData?.forEach((r) => map.set(r.jobId, r.score));
    return map;
  }, [rankData]);

  const jobs = useMemo(() => {
    const q = search.trim().toLowerCase();
    let list = q
      ? allJobs.filter(
          (j) =>
            j.title.toLowerCase().includes(q) ||
            j.company.toLowerCase().includes(q) ||
            j.description.toLowerCase().includes(q),
        )
      : [...allJobs];

    if (sortByScore && scoreById.size > 0) {
      list.sort((a, b) => (scoreById.get(b.id) ?? 0) - (scoreById.get(a.id) ?? 0));
    }
    return list;
  }, [allJobs, search, sortByScore, scoreById]);

  const { data: applications = [] } = useQuery<Application[]>({
    queryKey: queryKeys.applications.all(),
    queryFn: getApplications,
  });

  const { data: stats } = useQuery({
    queryKey: queryKeys.dashboard.stats(),
    queryFn: getDashboardStats,
  });

  const saveMutation = useMutation({
    mutationFn: async ({
      jobId,
      hasApplication,
    }: {
      jobId: string;
      hasApplication: boolean;
    }) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await markJobSaved(profileId, jobId);

      if (!hasApplication) {
        await createApplication({ jobId, status: 'saved' });
      }
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.applications.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.stats() });
      if (profileId) void queryClient.invalidateQueries({ queryKey: queryKeys.feedback.profile(profileId) });
      toast.success('Збережено в pipeline');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const hideMutation = useMutation({
    mutationFn: async (jobId: string) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await hideJobForProfile(profileId, jobId);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      if (profileId) void queryClient.invalidateQueries({ queryKey: queryKeys.feedback.profile(profileId) });
      toast.success('Вакансію приховано');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const badFitMutation = useMutation({
    mutationFn: async (jobId: string) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await markJobBadFit(profileId, jobId);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      if (profileId) void queryClient.invalidateQueries({ queryKey: queryKeys.feedback.profile(profileId) });
      toast.success('Позначено як bad fit');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const unmarkBadFitMutation = useMutation({
    mutationFn: async (jobId: string) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await unmarkJobBadFit(profileId, jobId);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      if (profileId) void queryClient.invalidateQueries({ queryKey: queryKeys.feedback.profile(profileId) });
      toast.success('Позначку bad fit знято');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const companyFeedbackMutation = useMutation({
    mutationFn: async ({
      company,
      currentStatus,
      nextStatus,
    }: {
      company: string;
      currentStatus?: 'whitelist' | 'blacklist';
      nextStatus: 'whitelist' | 'blacklist';
    }) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      if (nextStatus === 'whitelist') {
        if (currentStatus === 'whitelist') {
          await removeCompanyWhitelist(profileId, company);
        } else {
          await addCompanyWhitelist(profileId, company);
        }
        return;
      }

      if (currentStatus === 'blacklist') {
        await removeCompanyBlacklist(profileId, company);
      } else {
        await addCompanyBlacklist(profileId, company);
      }
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      if (profileId) void queryClient.invalidateQueries({ queryKey: queryKeys.feedback.profile(profileId) });
      toast.success('Оновлено список компанії');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const applicationByJob = new Map(applications.map((item) => [item.jobId, item]));
  const error =
    jobsError instanceof Error ? jobsError.message : jobsError ? 'Error' : null;

  useEffect(() => {
    void logJobImpressionsOnce({
      profileId,
      jobs,
      surface: 'dashboard_recent_jobs',
    });
  }, [jobs, profileId]);

  // Derived filter options for FilterChips
  const lifecycleOptions = LIFECYCLE_TABS.map((t) => ({ id: t.value, label: t.label }));
  const sourceOptions = [
    { id: '__all__', label: 'Всі джерела' },
    ...sources.map((s) => ({ id: s.id, label: s.displayName })),
  ];
  const selectedLifecycle = [lifecycleFilter];
  const selectedSource = sourceFilter ? [sourceFilter] : ['__all__'];
  const insights = [
    {
      id: 'active-feed',
      type: 'trend' as const,
      title: 'Feed health is stable',
      description: jobSummary
        ? `${jobSummary.activeJobs} active jobs are in rotation and ${jobSummary.reactivatedJobs} came back after disappearing.`
        : 'Track active and reactivated inventory from the dashboard feed.',
      action: { label: 'View analytics', href: '/analytics' },
    },
    {
      id: 'pipeline',
      type: 'recommendation' as const,
      title: 'Pipeline needs frequent review',
      description: applications.length > 0
        ? `${applications.length} tracked applications already affect ranking and next actions. Keep statuses current to improve feedback loops.`
        : 'Saved jobs and applications will appear here once you start tracking them.',
      action: { label: 'Open applications', href: '/applications' },
    },
    {
      id: 'profile',
      type: 'tip' as const,
      title: 'Search quality comes from profile quality',
      description: 'Refresh your profile and search preferences when target roles, regions, or allowed sources change.',
      action: { label: 'Update profile', href: '/profile' },
    },
  ];

  return (
    <Page>

      {/* ── Hero card ─────────────────────────────────────────────────────── */}
      <Card className="border-border bg-card overflow-hidden">
        <CardContent className="p-0">
          <div className="relative">
            <div className="absolute inset-0 bg-gradient-to-r from-primary/5 via-accent/5 to-transparent pointer-events-none" />
            <div className="relative p-6">
              <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-6">
                <div className="space-y-2">
                  <div className="flex items-center gap-2">
                    <Badge variant="default" className="bg-primary/15 text-primary border-0 text-xs px-2 py-0.5">
                      <Zap className="h-3 w-3 mr-1" />
                      Job Copilot UA
                    </Badge>
                  </div>
                  <h1 className="text-2xl font-bold text-card-foreground m-0">
                    Відстежуйте вакансії та заявки
                  </h1>
                  <p className="text-muted-foreground m-0">
                    Вакансії автоматично збираються з Djinni та Work.ua.{' '}
                    {jobSummary && (
                      <span className="text-primary font-medium">
                        {jobSummary.activeJobs} активних зараз.
                      </span>
                    )}
                  </p>
                </div>
                <div className="flex items-center gap-3">
                  <Link to="/profile">
                    <Button className="flex items-center gap-2">
                      <Sparkles className="h-4 w-4" />
                      Update Profile
                    </Button>
                  </Link>
                  <Link to="/applications">
                    <Button variant="ghost" className="flex items-center gap-2">
                      Review Pipeline
                      <ArrowRight size={14} />
                    </Button>
                  </Link>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {error && <p className="error">{error}</p>}

      {/* ── Stats grid ────────────────────────────────────────────────────── */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <StatCard
          title="Активних вакансій"
          value={jobSummary?.activeJobs ?? allJobs.length}
          description="зараз у базі"
          icon={Briefcase}
        />
        <StatCard
          title="Збережено"
          value={stats?.byStatus.saved ?? 0}
          icon={Bookmark}
        />
        <StatCard
          title="Подано заявки"
          value={stats?.byStatus.applied ?? 0}
          icon={Send}
        />
        <StatCard
          title="Інтерв'ю"
          value={(stats?.byStatus.interview ?? 0) + (stats?.byStatus.offer ?? 0)}
          icon={CalendarDays}
        />
      </div>

      {/* ── Main 2-col grid ───────────────────────────────────────────────── */}
      <div className="grid lg:grid-cols-3 gap-6">

        {/* Jobs column (2/3) */}
        <div className="lg:col-span-2 space-y-4">

          <SectionHeader
            title="Вакансії"
            description="Відсортовані за ML-скором або датою"
            icon={Briefcase}
          />

          {/* Filters */}
          <div className="space-y-2">
            <FilterChips
              options={lifecycleOptions}
              selected={selectedLifecycle}
              onChange={([v]) => updateFilters({ lifecycle: (v ?? 'all') as LifecycleFilter })}
            />
            <FilterChips
              options={sourceOptions}
              selected={selectedSource}
              onChange={([v]) => updateFilters({ source: v === '__all__' || !v ? null : v })}
            />
          </div>

          {/* Search + sort row */}
          <div className="flex items-center gap-3">
            <div className="relative flex-1">
              <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-content-muted pointer-events-none" />
              <input
                type="search"
                placeholder="Фільтр за назвою, компанією…"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                style={{ paddingLeft: 32 }}
              />
            </div>
            {rankData && rankData.length > 0 && (
              <Button
                variant={sortByScore ? 'default' : 'ghost'}
                size="sm"
                onClick={() => setSortByScore((v) => !v)}
                title="Сортувати за ML-релевантністю"
              >
                <SortAsc size={14} />
                Score
              </Button>
            )}
            {search && (
              <span className="text-muted-foreground text-xs shrink-0">
                {jobs.length}/{allJobs.length}
              </span>
            )}
          </div>

          {sourcesError && (
            <p className="text-muted-foreground text-xs">
              Каталог джерел недоступний — фільтр за джерелом тимчасово не працює.
            </p>
          )}

          {/* Job list */}
          <div className="space-y-3">
            {jobsLoading ? (
              <>
                <JobCardSkeleton />
                <JobCardSkeleton />
                <JobCardSkeleton />
              </>
            ) : jobs.length === 0 ? (
              <EmptyState
                message={search ? 'Нічого не знайдено' : 'Вакансій поки немає'}
                description={search ? 'Спробуйте змінити запит.' : 'Запустіть `pnpm scrape:djinni` або оновіть feed.'}
                icon={<Briefcase className="h-12 w-12" />}
              />
            ) : (
              jobs.map((job) => {
                const application = applicationByJob.get(job.id);
                const isSaved = !!(job.feedback?.saved || application);
                const isBadFit = !!job.feedback?.badFit;
                const isPendingAny = saveMutation.isPending || hideMutation.isPending || badFitMutation.isPending || unmarkBadFitMutation.isPending || companyFeedbackMutation.isPending;

                return (
                  <JobCard
                    key={job.id}
                    job={job}
                    score={scoreById.get(job.id)}
                    application={application}
                    isSaved={isSaved}
                    isBadFit={isBadFit}
                    isPending={isPendingAny}
                    onSave={!isSaved && !application ? () => saveMutation.mutate({ jobId: job.id, hasApplication: false }) : undefined}
                    onHide={() => hideMutation.mutate(job.id)}
                    onBadFit={!isBadFit ? () => badFitMutation.mutate(job.id) : undefined}
                    onUnmarkBadFit={isBadFit ? () => unmarkBadFitMutation.mutate(job.id) : undefined}
                  />
                );
              })
            )}
          </div>
        </div>

        {/* Sidebar (1/3) */}
        <div className="space-y-4">
          <AIInsightPanel insights={insights} />

          {/* Quick actions */}
          <Card className="border-border bg-card">
            <CardHeader>
              <h2 className="text-base font-semibold text-card-foreground m-0">Quick Actions</h2>
            </CardHeader>
            <CardContent className="space-y-2">
              <Link to="/profile" className="no-underline block">
                <Button variant="ghost" className="w-full justify-start gap-2 text-sm">
                  <Sparkles className="h-4 w-4 text-primary" />
                  Update Search Profile
                </Button>
              </Link>
              <Link to="/feedback" className="no-underline block">
                <Button variant="ghost" className="w-full justify-start gap-2 text-sm">
                  <Bookmark className="h-4 w-4 text-primary" />
                  Review Saved Jobs
                </Button>
              </Link>
              <Link to="/analytics" className="no-underline block">
                <Button variant="ghost" className="w-full justify-start gap-2 text-sm">
                  <TrendingUp className="h-4 w-4 text-primary" />
                  View Analytics
                </Button>
              </Link>
              <Link to="/applications" className="no-underline block">
                <Button variant="ghost" className="w-full justify-start gap-2 text-sm">
                  <KanbanSquare className="h-4 w-4 text-primary" />
                  Application Board
                </Button>
              </Link>
            </CardContent>
          </Card>

          {/* Pipeline summary */}
          {stats && (
            <Card className="border-border bg-card">
              <CardHeader>
                <h2 className="text-base font-semibold text-card-foreground m-0 flex items-center gap-2">
                  <XCircle className="h-4 w-4 text-primary" />
                  Pipeline
                </h2>
              </CardHeader>
              <CardContent className="space-y-3">
                {STATUS_COLUMNS.map((status) => (
                  <div key={status} className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground flex items-center gap-1.5">
                      {STATUS_ICONS[status]}
                      {status}
                    </span>
                    <span className="font-medium text-card-foreground">
                      {stats.byStatus[status] ?? 0}
                    </span>
                  </div>
                ))}
                <div className="flex items-center justify-between text-sm pt-2 border-t border-border">
                  <span className="text-muted-foreground">total tracked</span>
                  <span className="font-medium text-card-foreground">{applications.length}</span>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Feed summary */}
          {jobSummary && (
            <Card className="border-border bg-card">
              <CardHeader>
                <h2 className="text-base font-semibold text-card-foreground m-0">Feed</h2>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Active</span>
                  <span className="font-medium text-fit-excellent">{jobSummary.activeJobs}</span>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Reactivated</span>
                  <span className="font-medium text-fit-good">{jobSummary.reactivatedJobs}</span>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Inactive</span>
                  <span className="font-medium text-muted-foreground">{jobSummary.inactiveJobs}</span>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Total tracked</span>
                  <span className="font-medium text-card-foreground">{jobSummary.totalJobs}</span>
                </div>
              </CardContent>
            </Card>
          )}

        </div>
      </div>
    </Page>
  );
}
