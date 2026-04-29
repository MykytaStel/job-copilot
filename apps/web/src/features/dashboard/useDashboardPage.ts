import { useCallback, useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import type { LucideIcon } from 'lucide-react';
import { Bookmark, Briefcase, CalendarDays, Send, XCircle } from 'lucide-react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useToast } from '../../context/ToastContext';
import type {
  Application,
  ApplicationStatus,
  JobFeedSummary,
  JobPosting,
} from '@job-copilot/shared';

import { createApplication, getApplications, getDashboardStats } from '../../api/applications';
import {
  bulkHideJobsByCompany,
  hideJobForProfile,
  markJobBadFit,
  markJobSaved,
  undoJobBadFit,
  undoJobHide,
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
import {
  type SortMode,
  readPersistedLifecycle,
  writePersistedLifecycle,
  readPersistedSource,
  writePersistedSource,
} from '../../lib/displayPrefs';
import { useDisplayPrefs } from '../../lib/useDisplayPrefs';
import { queryKeys } from '../../queryKeys';

export type LifecycleFilter = 'all' | 'active' | 'inactive' | 'reactivated';

export const STATUS_COLUMNS: ApplicationStatus[] = [
  'saved',
  'applied',
  'interview',
  'offer',
  'rejected',
];

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
const DASHBOARD_RERANK_WINDOW = 60;
const UNDO_TOAST_DURATION_MS = 30_000;

function normalizeCompanyName(companyName: string) {
  return companyName.trim().replace(/\s+/g, ' ').toLowerCase();
}

function readLifecycleFilter(searchParams: URLSearchParams): LifecycleFilter {
  const lifecycle = searchParams.get('lifecycle');

  if (lifecycle === 'active' || lifecycle === 'inactive' || lifecycle === 'reactivated') {
    return lifecycle;
  }

  const persisted = readPersistedLifecycle();
  if (persisted === 'active' || persisted === 'inactive' || persisted === 'reactivated') {
    return persisted;
  }

  return DEFAULT_LIFECYCLE_FILTER;
}

function readSourceFilter(searchParams: URLSearchParams): string | null {
  const source = searchParams.get('source')?.trim();
  if (source) return source;
  return readPersistedSource();
}

function readJobIdFilter(searchParams: URLSearchParams): string[] {
  const raw = searchParams.get('job_ids')?.trim();
  if (!raw) return [];
  return Array.from(new Set(raw.split(',').map((value) => value.trim()).filter(Boolean)));
}

function readCompanyFilter(searchParams: URLSearchParams): string | null {
  return searchParams.get('company')?.trim() || null;
}

export function useDashboardPage() {
  const profileId = readProfileId();
  const queryClient = useQueryClient(); const { showToast } = useToast();
  const [searchParams, setSearchParams] = useSearchParams();
  const [search, setSearch] = useState('');
  const { sortMode, setSortMode: setDisplaySortMode } = useDisplayPrefs();
  const setSortMode = useCallback((next: SortMode) => {
    setDisplaySortMode(next);
  }, [setDisplaySortMode]);
  const lifecycleFilter = readLifecycleFilter(searchParams);
  const sourceFilter = readSourceFilter(searchParams);
  const notificationJobIds = readJobIdFilter(searchParams);
  const companyFilter = readCompanyFilter(searchParams);

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
        writePersistedLifecycle(null);
      } else {
        nextSearchParams.set('lifecycle', lifecycle);
        writePersistedLifecycle(lifecycle);
      }
    }

    if (source !== undefined) {
      if (source) {
        nextSearchParams.set('source', source);
        writePersistedSource(source);
      } else {
        nextSearchParams.delete('source');
        writePersistedSource(null);
      }
    }

    setSearchParams(nextSearchParams, { replace: true });
  };

  const clearContextFilters = useCallback(() => {
    const nextSearchParams = new URLSearchParams(searchParams);
    nextSearchParams.delete('job_ids');
    nextSearchParams.delete('company');
    setSearchParams(nextSearchParams, { replace: true });
  }, [searchParams, setSearchParams]);

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
  const rerankCandidates = useMemo(
    () => allJobs.slice(0, DASHBOARD_RERANK_WINDOW),
    [allJobs],
  );
  const rerankJobIds = rerankCandidates.map((job) => job.id);
  const rerankJobsKey = rerankJobIds.join('|');

  const { data: rankData } = useQuery<RankedJob[]>({
    queryKey: queryKeys.ml.rerank(profileId ?? '', rerankJobsKey),
    queryFn: () => rerankJobs(profileId!, rerankJobIds),
    enabled: !!profileId && sortMode === 'relevance' && rerankJobIds.length > 0,
    retry: false,
  });

  const scoreById = useMemo(() => {
    const map = new Map<string, number>();
    rankData?.forEach((item) => map.set(item.jobId, item.score));
    return map;
  }, [rankData]);

  const jobs = useMemo(() => {
    const normalized = search.trim().toLowerCase();
    let filtered = normalized
      ? allJobs.filter(
          (job) =>
            job.title.toLowerCase().includes(normalized) ||
            job.company.toLowerCase().includes(normalized) ||
            job.description.toLowerCase().includes(normalized),
        )
      : [...allJobs];

    if (notificationJobIds.length > 0) {
      const allowedIds = new Set(notificationJobIds);
      filtered = filtered.filter((job) => allowedIds.has(job.id));
    }

    if (companyFilter) {
      const normalizedCompany = normalizeCompanyName(companyFilter);
      filtered = filtered.filter((job) => normalizeCompanyName(job.company) === normalizedCompany);
    }

    if (sortMode === 'relevance' && scoreById.size > 0) {
      filtered.sort(
        (left, right) => (scoreById.get(right.id) ?? 0) - (scoreById.get(left.id) ?? 0),
      );
    } else if (sortMode === 'salary') {
      filtered.sort((left, right) => (right.salaryMin ?? 0) - (left.salaryMin ?? 0));
    }

    return filtered;
  }, [allJobs, companyFilter, notificationJobIds, scoreById, search, sortMode]);

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
      showToast({ type: 'success', message: 'Job saved' });
    },
    onError: (value: unknown) => {
      showToast({ type: 'error', message: value instanceof Error ? `Error: ${value.message}` : 'Error: action failed' });
    },
  });

  const undoHideMutation = useMutation({
    mutationFn: async (jobId: string) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await undoJobHide(profileId, jobId);
    },
    onSuccess: () => {
      void invalidateFeedbackViewQueries(queryClient, profileId);
      showToast({ type: 'success', message: 'Hide undone' });
    },
    onError: (value: unknown) => {
      showToast({ type: 'error', message: value instanceof Error ? `Error: ${value.message}` : 'Error: action failed' });
    },
  });

  const undoBadFitMutation = useMutation({
    mutationFn: async (jobId: string) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await undoJobBadFit(profileId, jobId);
    },
    onSuccess: () => {
      void invalidateFeedbackViewQueries(queryClient, profileId);
      showToast({ type: 'success', message: 'Bad-fit mark undone' });
    },
    onError: (value: unknown) => {
      showToast({ type: 'error', message: value instanceof Error ? `Error: ${value.message}` : 'Error: action failed' });
    },
  });

  const hideMutation = useMutation({
    mutationFn: async (jobId: string) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await hideJobForProfile(profileId, jobId);
    },
    onSuccess: (_result, jobId) => {
      void invalidateFeedbackViewQueries(queryClient, profileId);
      showToast({
        type: 'success',
        message: 'Job hidden',
        action: {
          label: 'Undo',
          onClick: () => undoHideMutation.mutate(jobId),
        },
        durationMs: UNDO_TOAST_DURATION_MS,
      });
    },
    onError: (value: unknown) => {
      showToast({ type: 'error', message: value instanceof Error ? `Error: ${value.message}` : 'Error: action failed' });
    },
  });

  const bulkHideCompanyMutation = useMutation({
    mutationFn: async (companyName: string) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      const normalizedCompany = normalizeCompanyName(companyName);
      const affectedInFeed = allJobs.filter(
        (job) => normalizeCompanyName(job.company) === normalizedCompany && !job.feedback?.hidden,
      ).length;

      if (
        affectedInFeed > 5 &&
        !window.confirm(`Hide ${affectedInFeed} jobs from ${companyName}?`)
      ) {
        return null;
      }

      return bulkHideJobsByCompany(profileId, companyName);
    },
    onSuccess: (result) => {
      if (!result) return;
      void invalidateFeedbackViewQueries(queryClient, profileId);
      showToast({
        type: 'success',
        message: `Hidden ${result.affectedCount} jobs from this company`,
      });
    },
    onError: (value: unknown) => {
      showToast({ type: 'error', message: value instanceof Error ? `Error: ${value.message}` : 'Error: action failed' });
    },
  });

  const badFitMutation = useMutation({
    mutationFn: async ({ jobId, reason }: { jobId: string; reason?: string }) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await markJobBadFit(profileId, jobId, reason);
    },
    onSuccess: (_result, variables) => {
      void invalidateFeedbackViewQueries(queryClient, profileId);
      showToast({
        type: 'success',
        message: 'Marked as bad fit',
        action: {
          label: 'Undo',
          onClick: () => undoBadFitMutation.mutate(variables.jobId),
        },
        durationMs: UNDO_TOAST_DURATION_MS,
      });
    },
    onError: (value: unknown) => {
      showToast({ type: 'error', message: value instanceof Error ? `Error: ${value.message}` : 'Error: action failed' });
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
      showToast({ type: 'success', message: 'Bad-fit mark removed' });
    },
    onError: (value: unknown) => {
      showToast({ type: 'error', message: value instanceof Error ? `Error: ${value.message}` : 'Error: action failed' });
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
  const mode = sortMode === 'relevance' && profileId ? 'ranked' : 'recent';
  const rerankCoverage = {
    rankedJobs: rerankJobIds.length,
    totalJobs: allJobs.length,
    isTruncated: allJobs.length > rerankJobIds.length,
  };

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
    sortMode,
    setSortMode,
    lifecycleFilter,
    sourceFilter,
    notificationJobIds,
    companyFilter,
    clearContextFilters,
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
    rerankCoverage,
    insights,
    saveMutation,
    hideMutation,
    undoHideMutation,
    bulkHideCompanyMutation,
    badFitMutation,
    undoBadFitMutation,
    unmarkBadFitMutation,
  };
}

export type DashboardPageState = ReturnType<typeof useDashboardPage>;
