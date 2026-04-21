import { useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';

import {
  addCompanyBlacklist,
  addCompanyWhitelist,
  getFeedback,
  removeCompanyBlacklist,
  removeCompanyWhitelist,
  unhideJob,
  unmarkJobBadFit,
  unsaveJob,
} from '../../api/feedback';
import { getJobsFeed } from '../../api/jobs';
import { invalidateFeedbackViewQueries } from '../../lib/queryInvalidation';
import { readProfileId } from '../../lib/profileSession';
import { queryKeys } from '../../queryKeys';
import {
  FEEDBACK_TAB_META,
  type FeedbackTab,
} from './FeedbackCenterComponents';
import { filterJobsBySearch } from './FeedbackCenterSections';

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

  const tabCounts: Record<FeedbackTab, number> = {
    saved: savedJobs.length,
    hidden: hiddenJobs.length,
    'bad-fit': badFitJobs.length,
    companies: whitelistedCompanies.length + blacklistedCompanies.length,
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
    tabCounts,
    activeTabMeta,
    filteredSavedJobs,
    filteredHiddenJobs,
    filteredBadFitJobs,
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
    submitCompany,
  };
}

export type FeedbackCenterPageState = ReturnType<typeof useFeedbackCenterPage>;
