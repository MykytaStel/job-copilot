import { useMemo, useState } from 'react';
import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';

import {
  addCompanyBlacklist,
  addCompanyWhitelist,
  bulkHideJobsByCompany,
  exportFeedback,
  type FeedbackExportType,
  getFeedback,
  getFeedbackStats,
  getFeedbackTimeline,
  removeCompanyBlacklist,
  removeCompanyWhitelist,
  unhideJob,
  unmarkJobBadFit,
  unsaveJob,
  updateCompanyFeedbackNotes,
} from '../../api/feedback';
import { getJobsFeed } from '../../api/jobs';
import { invalidateFeedbackViewQueries } from '../../lib/queryInvalidation';
import { readProfileId } from '../../lib/profileSession';
import { queryKeys } from '../../queryKeys';
import { FEEDBACK_TAB_META, type FeedbackTab } from './FeedbackCenterComponents';
import { filterJobsBySearch } from './FeedbackCenterSections';

const TIMELINE_PAGE_SIZE = 20;

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

  function invalidateAfterAction() {
    void invalidateFeedbackViewQueries(queryClient, profileId);
  }

  const unsaveMutation = useMutation({
    mutationFn: (jobId: string) => unsaveJob(profileId!, jobId),
    onSuccess: () => {
      invalidateAfterAction();
      toast.success('Збережено скасовано');
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const unhideMutation = useMutation({
    mutationFn: (jobId: string) => unhideJob(profileId!, jobId),
    onSuccess: () => {
      invalidateAfterAction();
      toast.success('Вакансію показано знову');
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const unmarkBadFitMutation = useMutation({
    mutationFn: (jobId: string) => unmarkJobBadFit(profileId!, jobId),
    onSuccess: () => {
      invalidateAfterAction();
      toast.success('Позначку bad fit знято');
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const removeWhitelistMutation = useMutation({
    mutationFn: (companyName: string) => removeCompanyWhitelist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterAction();
      toast.success('Видалено зі списку дозволених');
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const removeBlacklistMutation = useMutation({
    mutationFn: (companyName: string) => removeCompanyBlacklist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterAction();
      toast.success('Видалено зі списку заблокованих');
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const addWhitelistMutation = useMutation({
    mutationFn: (companyName: string) => addCompanyWhitelist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterAction();
      setWhitelistInput('');
      toast.success('Компанію додано до whitelist');
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const addBlacklistMutation = useMutation({
    mutationFn: (companyName: string) => addCompanyBlacklist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterAction();
      setBlacklistInput('');
      toast.success('Компанію додано до blacklist');
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Помилка'),
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
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const updateCompanyNotesMutation = useMutation({
    mutationFn: ({ companySlug, notes }: { companySlug: string; notes: string }) =>
      updateCompanyFeedbackNotes(profileId!, companySlug, notes),
    onSuccess: () => {
      invalidateAfterAction();
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Помилка'),
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
      invalidateAfterAction();
      toast.success(`Hidden ${result.affectedCount} jobs from this company`);
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Помилка'),
  });

  const exportMutation = useMutation({
    mutationFn: async () => exportFeedback(exportTypeForTab(activeTab)),
    onSuccess: ({ blob, filename }) => {
      downloadBlob(blob, filename ?? `feedback-${exportTypeForTab(activeTab)}.csv`);
      toast.success('Feedback CSV exported');
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Помилка'),
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
    setActiveTab,
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
