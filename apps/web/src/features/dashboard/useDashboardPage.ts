import { useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import type { LucideIcon } from 'lucide-react';
import { Bookmark, Briefcase, CalendarDays, Send, XCircle } from 'lucide-react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type {
  Application,
  ApplicationStatus,
  JobFeedSummary,
  JobPosting,
} from '@job-copilot/shared';

import { createApplication, getApplications, getDashboardStats } from '../../api/applications';
import {
  hideJobForProfile,
  markJobBadFit,
  markJobSaved,
  unmarkJobBadFit,
} from '../../api/feedback';
import type { RankedJob } from '../../api/jobs';
import { getJobsFeed, rerankJobs } from '../../api/jobs';
import { getSources } from '../../api/profiles';
import { logJobImpressionsOnce } from '../events/jobImpressions';
import {
  invalidateApplicationSummaryQueries,
  invalidateFeedbackViewQueries,
} from '../../lib/queryInvalidation';
import { readProfileId } from '../../lib/profileSession';
import { queryKeys } from '../../queryKeys';

export type LifecycleFilter = 'all' | 'active' | 'inactive' | 'reactivated';

export const STATUS_COLUMNS: ApplicationStatus[] = ['saved', 'applied', 'interview', 'offer', 'rejected'];

export const STATUS_ICONS = {
  saved: Bookmark,
  applied: Send,
  interview: CalendarDays,
  offer: Briefcase,
  rejected: XCircle,
} satisfies Record<ApplicationStatus, LucideIcon>;

const LIFECYCLE_TABS: { value: LifecycleFilter; label: string }[] = [
  { value: 'all', label: 'Всі' },
  { value: 'active', label: 'Активні' },
  { value: 'inactive', label: 'Зникли' },
  { value: 'reactivated', label: 'Повернулись' },
];

const DEFAULT_LIFECYCLE_FILTER: LifecycleFilter = 'all';

function readLifecycleFilter(searchParams: URLSearchParams): LifecycleFilter {
  const lifecycle = searchParams.get('lifecycle');

  if (lifecycle === 'active' || lifecycle === 'inactive' || lifecycle === 'reactivated') {
    return lifecycle;
  }

  return DEFAULT_LIFECYCLE_FILTER;
}

function readSourceFilter(searchParams: URLSearchParams): string | null {
  const source = searchParams.get('source')?.trim();
  return source ? source : null;
}

export function useDashboardPage() {
  const profileId = readProfileId();
  const queryClient = useQueryClient();
  const [searchParams, setSearchParams] = useSearchParams();
  const [search, setSearch] = useState('');
  const [sortByScore, setSortByScore] = useState(false);
  const lifecycleFilter = readLifecycleFilter(searchParams);
  const sourceFilter = readSourceFilter(searchParams);

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

  const {
    data: jobsFeed,
    error: jobsError,
    isLoading: jobsLoading,
  } = useQuery<{
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

  const { data: sources = [], error: sourcesError } = useQuery({
    queryKey: queryKeys.sources.all(),
    queryFn: getSources,
    staleTime: 5 * 60_000,
  });

  const allJobs = useMemo(() => jobsFeed?.jobs ?? [], [jobsFeed?.jobs]);
  const jobSummary = jobsFeed?.summary;
  const rerankJobIds = allJobs.map((job) => job.id);
  const rerankJobsKey = rerankJobIds.join('|');

  const { data: rankData } = useQuery<RankedJob[]>({
    queryKey: queryKeys.ml.rerank(profileId ?? '', rerankJobsKey),
    queryFn: () => rerankJobs(profileId!, rerankJobIds),
    enabled: !!profileId && allJobs.length > 0,
    retry: false,
  });

  const scoreById = useMemo(() => {
    const map = new Map<string, number>();
    rankData?.forEach((item) => map.set(item.jobId, item.score));
    return map;
  }, [rankData]);

  const jobs = useMemo(() => {
    const normalized = search.trim().toLowerCase();
    const filtered = normalized
      ? allJobs.filter(
          (job) =>
            job.title.toLowerCase().includes(normalized) ||
            job.company.toLowerCase().includes(normalized) ||
            job.description.toLowerCase().includes(normalized),
        )
      : [...allJobs];

    if (sortByScore && scoreById.size > 0) {
      filtered.sort((left, right) => (scoreById.get(right.id) ?? 0) - (scoreById.get(left.id) ?? 0));
    }

    return filtered;
  }, [allJobs, scoreById, search, sortByScore]);

  const { data: applications = [] } = useQuery<Application[]>({
    queryKey: queryKeys.applications.all(),
    queryFn: getApplications,
  });

  const { data: stats } = useQuery({
    queryKey: queryKeys.dashboard.stats(),
    queryFn: getDashboardStats,
  });

  const saveMutation = useMutation({
    mutationFn: async ({ jobId, hasApplication }: { jobId: string; hasApplication: boolean }) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await markJobSaved(profileId, jobId);

      if (!hasApplication) {
        await createApplication({ jobId, status: 'saved' });
      }
    },
    onSuccess: () => {
      void invalidateFeedbackViewQueries(queryClient, profileId);
      void invalidateApplicationSummaryQueries(queryClient);
      toast.success('Збережено в pipeline');
    },
    onError: (value: unknown) => {
      toast.error(value instanceof Error ? value.message : 'Помилка');
    },
  });

  const hideMutation = useMutation({
    mutationFn: async (jobId: string) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await hideJobForProfile(profileId, jobId);
    },
    onSuccess: () => {
      void invalidateFeedbackViewQueries(queryClient, profileId);
      toast.success('Вакансію приховано');
    },
    onError: (value: unknown) => {
      toast.error(value instanceof Error ? value.message : 'Помилка');
    },
  });

  const badFitMutation = useMutation({
    mutationFn: async (jobId: string) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await markJobBadFit(profileId, jobId);
    },
    onSuccess: () => {
      void invalidateFeedbackViewQueries(queryClient, profileId);
      toast.success('Позначено як bad fit');
    },
    onError: (value: unknown) => {
      toast.error(value instanceof Error ? value.message : 'Помилка');
    },
  });

  const unmarkBadFitMutation = useMutation({
    mutationFn: async (jobId: string) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await unmarkJobBadFit(profileId, jobId);
    },
    onSuccess: () => {
      void invalidateFeedbackViewQueries(queryClient, profileId);
      toast.success('Позначку bad fit знято');
    },
    onError: (value: unknown) => {
      toast.error(value instanceof Error ? value.message : 'Помилка');
    },
  });

  const applicationByJob = useMemo(
    () => new Map(applications.map((item) => [item.jobId, item])),
    [applications],
  );
  const error = jobsError instanceof Error ? jobsError.message : jobsError ? 'Error' : null;

  useEffect(() => {
    void logJobImpressionsOnce({
      profileId,
      jobs,
      surface: 'dashboard_recent_jobs',
    });
  }, [jobs, profileId]);

  const lifecycleOptions = LIFECYCLE_TABS.map((tab) => ({
    id: tab.value,
    label: tab.label,
  }));
  const sourceOptions = [
    { id: '__all__', label: 'Всі джерела' },
    ...sources.map((source) => ({ id: source.id, label: source.displayName })),
  ];
  const selectedLifecycle = [lifecycleFilter];
  const selectedSource = sourceFilter ? [sourceFilter] : ['__all__'];
  const topSource =
    sources.find((source) => source.id === sourceFilter)?.displayName ?? 'All sources';
  const interviewedCount = (stats?.byStatus.interview ?? 0) + (stats?.byStatus.offer ?? 0);
  const mode = sortByScore ? 'ranked' : 'recent';

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
      description:
        applications.length > 0
          ? `${applications.length} tracked applications already affect ranking and next actions. Keep statuses current to improve feedback loops.`
          : 'Saved jobs and applications will appear here once you start tracking them.',
      action: { label: 'Open applications', href: '/applications' },
    },
    {
      id: 'profile',
      type: 'tip' as const,
      title: 'Search quality comes from profile quality',
      description:
        'Refresh your profile and search preferences when target roles, regions, or allowed sources change.',
      action: { label: 'Update profile', href: '/profile' },
    },
  ];

  return {
    profileId,
    search,
    setSearch,
    sortByScore,
    setSortByScore,
    lifecycleFilter,
    sourceFilter,
    updateFilters,
    jobsLoading,
    jobs,
    allJobs,
    jobSummary,
    sourcesError,
    applications,
    stats,
    rankData,
    scoreById,
    applicationByJob,
    error,
    lifecycleOptions,
    sourceOptions,
    selectedLifecycle,
    selectedSource,
    topSource,
    interviewedCount,
    mode,
    insights,
    saveMutation,
    hideMutation,
    badFitMutation,
    unmarkBadFitMutation,
  };
}

export type DashboardPageState = ReturnType<typeof useDashboardPage>;
