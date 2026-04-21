import { useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Search } from 'lucide-react';
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
} from '../api/feedback';
import { getJobsFeed } from '../api/jobs';
import { Badge } from '../components/ui/Badge';
import { Card, CardContent } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { StatCard } from '../components/ui/StatCard';
import { invalidateFeedbackViewQueries } from '../lib/queryInvalidation';
import { cn } from '../lib/cn';
import { queryKeys } from '../queryKeys';
import {
  FEEDBACK_SUMMARY_CARDS,
  FEEDBACK_TAB_META,
  type FeedbackTab,
} from './feedback/FeedbackCenterComponents';
import {
  BadFitJobsSection,
  CompaniesSection,
  filterJobsBySearch,
  HiddenJobsSection,
  SavedJobsSection,
} from './feedback/FeedbackCenterSections';

function readProfileId() {
  return window.localStorage.getItem('engine_api_profile_id');
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
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Feedback' }]}
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

          {summary ? (
            <div className="grid grid-cols-2 gap-4 lg:grid-cols-5">
              {FEEDBACK_SUMMARY_CARDS.map((item) => (
                <StatCard
                  key={item.key}
                  title={item.title}
                  value={summary[item.key]}
                  icon={item.icon}
                />
              ))}
            </div>
          ) : null}

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
                  {activeTab !== 'companies' ? (
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
                  ) : null}
                </div>
              </div>

              {activeTab === 'saved' ? (
                <SavedJobsSection
                  jobs={filteredSavedJobs}
                  searchQuery={searchQuery}
                  onUnsave={(jobId) => unsaveMutation.mutate(jobId)}
                  isPending={unsaveMutation.isPending}
                />
              ) : null}

              {activeTab === 'hidden' ? (
                <HiddenJobsSection
                  jobs={filteredHiddenJobs}
                  searchQuery={searchQuery}
                  onUnhide={(jobId) => unhideMutation.mutate(jobId)}
                  isPending={unhideMutation.isPending}
                />
              ) : null}

              {activeTab === 'bad-fit' ? (
                <BadFitJobsSection
                  jobs={filteredBadFitJobs}
                  searchQuery={searchQuery}
                  onUnmark={(jobId) => unmarkBadFitMutation.mutate(jobId)}
                  isPending={unmarkBadFitMutation.isPending}
                />
              ) : null}

              {activeTab === 'companies' ? (
                <CompaniesSection
                  whitelistedCompanies={whitelistedCompanies}
                  blacklistedCompanies={blacklistedCompanies}
                  whitelistInput={whitelistInput}
                  blacklistInput={blacklistInput}
                  onWhitelistInputChange={setWhitelistInput}
                  onBlacklistInputChange={setBlacklistInput}
                  onSubmitCompany={submitCompany}
                  onMoveCompany={(companyName, nextStatus) =>
                    moveCompanyMutation.mutate({ companyName, nextStatus })
                  }
                  onRemoveWhitelist={(companyName) => removeWhitelistMutation.mutate(companyName)}
                  onRemoveBlacklist={(companyName) => removeBlacklistMutation.mutate(companyName)}
                  isAddWhitelistPending={addWhitelistMutation.isPending}
                  isAddBlacklistPending={addBlacklistMutation.isPending}
                  isMovePending={moveCompanyMutation.isPending}
                  isRemoveWhitelistPending={removeWhitelistMutation.isPending}
                  isRemoveBlacklistPending={removeBlacklistMutation.isPending}
                />
              ) : null}
            </CardContent>
          </Card>
        </>
      )}
    </Page>
  );
}
