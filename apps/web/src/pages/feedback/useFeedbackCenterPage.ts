import { useMemo, useState } from 'react';
import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { useToast } from '../../context/ToastContext';

import {
  addCompanyBlacklist,
  addCompanyWhitelist,
  bulkHideJobsByCompany,
  exportFeedback,
  type FeedbackExportType,
  getFeedback,
  getFeedbackStats,
  getFeedbackTimeline,
  hideJobForProfile,
  markJobBadFit,
  markJobSaved,
  removeCompanyBlacklist,
  removeCompanyWhitelist,
  unhideJob,
  unmarkJobBadFit,
  unsaveJob,
  updateCompanyFeedbackNotes,
} from '../../api/feedback';
import { getJobsFeed } from '../../api/jobs';
import { invalidateFeedbackQueries, invalidateJobQueries } from '../../lib/queryInvalidation';
import { readProfileId } from '../../lib/profileSession';
import { queryKeys } from '../../queryKeys';
import { FEEDBACK_TAB_META, type FeedbackTab } from './FeedbackCenterComponents';
import { filterJobsBySearch } from './FeedbackCenterSections';

const TIMELINE_PAGE_SIZE = 20;

type BulkFeedbackAction = 'hide' | 'bad-fit' | 'save';

function normalizeCompanyName(companyName: string) {
  return companyName.trim().replace(/\s+/g, ' ').toLowerCase();
}

function exportTypeForTab(tab: FeedbackTab): FeedbackExportType {
  if (tab === 'timeline') return 'saved';
  return tab === 'bad-fit' ? 'bad_fit' : tab;
}

function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');

  anchor.href = url;
  anchor.download = filename;
  anchor.click();

  URL.revokeObjectURL(url);
}

export function useFeedbackCenterPage() {
  const profileId = readProfileId();
  const queryClient = useQueryClient();
  const { showToast } = useToast();
  const [activeTab, setActiveTab] = useState<FeedbackTab>('saved');
  const [searchQuery, setSearchQuery] = useState('');
  const [whitelistInput, setWhitelistInput] = useState('');
  const [blacklistInput, setBlacklistInput] = useState('');
  const [selectedJobIds, setSelectedJobIds] = useState<Set<string>>(() => new Set());

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

  const { data: feedbackStats } = useQuery({
    queryKey: queryKeys.feedback.stats(profileId ?? ''),
    queryFn: () => getFeedbackStats(profileId!),
    enabled: !!profileId,
    retry: false,
  });

  const feedbackTimelineQuery = useInfiniteQuery({
    queryKey: queryKeys.feedback.timeline(profileId ?? ''),
    queryFn: ({ pageParam }) =>
      getFeedbackTimeline(profileId!, { limit: TIMELINE_PAGE_SIZE, offset: pageParam }),
    initialPageParam: 0,
    getNextPageParam: (lastPage) => lastPage.nextOffset ?? undefined,
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

  const filteredSavedJobs = useMemo(
    () => filterJobsBySearch(savedJobs, normalizedSearch),
    [normalizedSearch, savedJobs],
  );
  const filteredHiddenJobs = useMemo(
    () => filterJobsBySearch(hiddenJobs, normalizedSearch),
    [normalizedSearch, hiddenJobs],
  );
  const filteredBadFitJobs = useMemo(
    () => filterJobsBySearch(badFitJobs, normalizedSearch),
    [normalizedSearch, badFitJobs],
  );
  const timelineItems = useMemo(
    () => feedbackTimelineQuery.data?.pages.flatMap((page) => page.items) ?? [],
    [feedbackTimelineQuery.data],
  );
  const timelineTotalCount = feedbackTimelineQuery.data?.pages[0]?.totalCount ?? 0;

  const tabCounts: Record<FeedbackTab, number> = {
    saved: savedJobs.length,
    hidden: hiddenJobs.length,
    'bad-fit': badFitJobs.length,
    companies: whitelistedCompanies.length + blacklistedCompanies.length,
    timeline: timelineTotalCount,
  };

  const activeTabMeta =
    FEEDBACK_TAB_META.find((tab) => tab.id === activeTab) ?? FEEDBACK_TAB_META[0];
  const activeBulkJobs =
    activeTab === 'saved'
      ? filteredSavedJobs
      : activeTab === 'hidden'
        ? filteredHiddenJobs
        : activeTab === 'bad-fit'
          ? filteredBadFitJobs
          : [];
  const activeBulkJobIds = useMemo(() => activeBulkJobs.map((job) => job.id), [activeBulkJobs]);
  const visibleSelectedJobIds = activeBulkJobIds.filter((jobId) => selectedJobIds.has(jobId));
  const selectedJobsCount = visibleSelectedJobIds.length;
  const allVisibleJobsSelected =
    activeBulkJobIds.length > 0 && activeBulkJobIds.every((jobId) => selectedJobIds.has(jobId));

  function invalidateAfterJobAction() {
    void invalidateJobQueries(queryClient, profileId);
  }

  function invalidateAfterFeedbackAction() {
    void invalidateFeedbackQueries(queryClient, profileId);
  }

  function toggleJobSelected(jobId: string) {
    setSelectedJobIds((current) => {
      const next = new Set(current);
      if (next.has(jobId)) {
        next.delete(jobId);
      } else {
        next.add(jobId);
      }
      return next;
    });
  }

  function selectAllVisibleJobs() {
    setSelectedJobIds((current) => {
      const next = new Set(current);
      activeBulkJobIds.forEach((jobId) => next.add(jobId));
      return next;
    });
  }

  function clearSelectedJobs() {
    setSelectedJobIds((current) => {
      if (current.size === 0) return current;
      return new Set();
    });
  }

  function setAllVisibleJobsSelected(isSelected: boolean) {
    if (isSelected) {
      selectAllVisibleJobs();
      return;
    }

    setSelectedJobIds((current) => {
      const next = new Set(current);
      activeBulkJobIds.forEach((jobId) => next.delete(jobId));
      return next;
    });
  }

  const unsaveMutation = useMutation({
    mutationFn: (jobId: string) => unsaveJob(profileId!, jobId),
    onSuccess: () => {
      invalidateAfterJobAction();
      showToast({ type: 'success', message: 'Збережено скасовано' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Помилка' }),
  });

  const unhideMutation = useMutation({
    mutationFn: (jobId: string) => unhideJob(profileId!, jobId),
    onSuccess: () => {
      invalidateAfterJobAction();
      showToast({ type: 'success', message: 'Вакансію показано знову' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Помилка' }),
  });

  const unmarkBadFitMutation = useMutation({
    mutationFn: (jobId: string) => unmarkJobBadFit(profileId!, jobId),
    onSuccess: () => {
      invalidateAfterJobAction();
      showToast({ type: 'success', message: 'Позначку bad fit знято' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Помилка' }),
  });

  const bulkJobActionMutation = useMutation({
    mutationFn: async (action: BulkFeedbackAction) => {
      const jobIds = visibleSelectedJobIds;
      if (jobIds.length === 0) return { action, count: 0 };

      if (action === 'hide') {
        await Promise.all(jobIds.map((jobId) => hideJobForProfile(profileId!, jobId)));
      } else if (action === 'bad-fit') {
        await Promise.all(jobIds.map((jobId) => markJobBadFit(profileId!, jobId)));
      } else {
        await Promise.all(jobIds.map((jobId) => markJobSaved(profileId!, jobId)));
      }

      return { action, count: jobIds.length };
    },
    onSuccess: ({ action, count }) => {
      if (count === 0) return;
      clearSelectedJobs();
      invalidateAfterJobAction();
      const label =
        action === 'hide'
          ? 'hidden'
          : action === 'bad-fit'
            ? 'marked as bad fit'
            : 'saved';
      showToast({ type: 'success', message: `${count} job${count === 1 ? '' : 's'} ${label}` });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Помилка' }),
  });

  const removeWhitelistMutation = useMutation({
    mutationFn: (companyName: string) => removeCompanyWhitelist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterFeedbackAction();
      showToast({ type: 'success', message: 'Видалено зі списку дозволених' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Помилка' }),
  });

  const removeBlacklistMutation = useMutation({
    mutationFn: (companyName: string) => removeCompanyBlacklist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterFeedbackAction();
      showToast({ type: 'success', message: 'Видалено зі списку заблокованих' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Помилка' }),
  });

  const addWhitelistMutation = useMutation({
    mutationFn: (companyName: string) => addCompanyWhitelist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterFeedbackAction();
      setWhitelistInput('');
      showToast({ type: 'success', message: 'Компанію додано до whitelist' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Помилка' }),
  });

  const addBlacklistMutation = useMutation({
    mutationFn: (companyName: string) => addCompanyBlacklist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterFeedbackAction();
      setBlacklistInput('');
      showToast({ type: 'success', message: 'Компанію додано до blacklist' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Помилка' }),
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
      invalidateAfterFeedbackAction();
      showToast({ type: 'success', message: 'Статус компанії оновлено' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Помилка' }),
  });

  const updateCompanyNotesMutation = useMutation({
    mutationFn: ({ companySlug, notes }: { companySlug: string; notes: string }) =>
      updateCompanyFeedbackNotes(profileId!, companySlug, notes),
    onSuccess: () => {
      invalidateAfterFeedbackAction();
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Помилка' }),
  });

  const bulkHideCompanyMutation = useMutation({
    mutationFn: async (companyName: string) => {
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

      return bulkHideJobsByCompany(profileId!, companyName);
    },
    onSuccess: (result) => {
      if (!result) return;
      invalidateAfterJobAction();
      showToast({ type: 'success', message: `Hidden ${result.affectedCount} jobs from this company` });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Помилка' }),
  });

  const exportMutation = useMutation({
    mutationFn: async () => exportFeedback(exportTypeForTab(activeTab)),
    onSuccess: ({ blob, filename }) => {
      downloadBlob(blob, filename ?? `feedback-${exportTypeForTab(activeTab)}.csv`);
      showToast({ type: 'success', message: 'Feedback CSV exported' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Помилка' }),
  });

  function loadMoreTimeline() {
    if (feedbackTimelineQuery.hasNextPage && !feedbackTimelineQuery.isFetchingNextPage) {
      void feedbackTimelineQuery.fetchNextPage();
    }
  }

  function submitCompany(status: 'whitelist' | 'blacklist') {
    const value = (status === 'whitelist' ? whitelistInput : blacklistInput).trim();
    if (!value) {
      return;
    }

    if (status === 'whitelist') {
      addWhitelistMutation.mutate(value);
      return;
    }

    addBlacklistMutation.mutate(value);
  }

  return {
    profileId,
    activeTab,
    setActiveTab: (tab: FeedbackTab) => {
      setActiveTab(tab);
      clearSelectedJobs();
    },
    searchQuery,
    setSearchQuery,
    whitelistInput,
    setWhitelistInput,
    blacklistInput,
    setBlacklistInput,
    savedJobs,
    badFitJobs,
    summary,
    feedbackStats,
    tabCounts,
    activeTabMeta,
    filteredSavedJobs,
    filteredHiddenJobs,
    filteredBadFitJobs,
    selectedJobIds,
    selectedJobsCount,
    allVisibleJobsSelected,
    setAllVisibleJobsSelected,
    toggleJobSelected,
    clearSelectedJobs,
    timelineItems,
    timelineTotalCount,
    hasMoreTimeline: feedbackTimelineQuery.hasNextPage,
    isTimelineLoading: feedbackTimelineQuery.isLoading,
    isTimelineLoadingMore: feedbackTimelineQuery.isFetchingNextPage,
    whitelistedCompanies,
    blacklistedCompanies,
    isLoading: jobsLoading || feedbackLoading,
    unsaveMutation,
    unhideMutation,
    unmarkBadFitMutation,
    bulkJobActionMutation,
    removeWhitelistMutation,
    removeBlacklistMutation,
    addWhitelistMutation,
    addBlacklistMutation,
    moveCompanyMutation,
    updateCompanyNotesMutation,
    bulkHideCompanyMutation,
    exportMutation,
    loadMoreTimeline,
    submitCompany,
  };
}

export type FeedbackCenterPageState = ReturnType<typeof useFeedbackCenterPage>;
