import { useMemo, useState } from 'react';
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
import { queryKeys } from '../queryKeys';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card } from '../components/ui/Card';
import { StatCard } from '../components/ui/StatCard';
import { PageHeader } from '../components/ui/SectionHeader';
import { cn } from '../lib/cn';

function readProfileId() {
  return window.localStorage.getItem('engine_api_profile_id');
}

type FeedbackTab = 'saved' | 'hidden' | 'bad-fit' | 'companies';

function JobRow({
  job,
  actionLabel,
  onAction,
  isPending,
}: {
  job: JobPosting;
  actionLabel: string;
  onAction: (jobId: string) => void;
  isPending: boolean;
}) {
  return (
    <Card className="flex items-center justify-between gap-4 px-4 py-3">
      <div className="min-w-0">
        <div className="text-sm font-semibold text-content truncate">
          {job.presentation?.title ?? job.title}
        </div>
        <div className="text-xs text-content-muted mt-0.5">
          {job.presentation?.company ?? job.company}
        </div>
      </div>
      <Button variant="ghost" size="sm" onClick={() => onAction(job.id)} disabled={isPending}>
        <Undo2 size={12} />
        {actionLabel}
      </Button>
    </Card>
  );
}

function Section({
  title,
  icon,
  children,
  count,
}: {
  title: string;
  icon: React.ReactNode;
  children: React.ReactNode;
  count: number;
  }) {
  return (
    <Card className="p-6">
      <div className="flex items-center gap-2 mb-4">
        <span className="text-content-muted">{icon}</span>
        <h2 className="m-0 text-[15px] font-semibold text-content">{title}</h2>
        <Badge variant="muted" className="ml-auto text-xs px-2 py-0.5 rounded-lg">{count}</Badge>
      </div>
      {children}
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
    queryFn: () => getJobsFeed({ limit: 500 }),
    enabled: !!profileId,
  });

  const { data: feedbackOverview, isLoading: feedbackLoading } = useQuery({
    queryKey: queryKeys.feedback.profile(profileId ?? ''),
    queryFn: () => getFeedback(profileId!),
    enabled: !!profileId,
  });

  const allJobs = jobsFeed?.jobs ?? [];
  const savedJobs = allJobs.filter((j) => j.feedback?.saved);
  const hiddenJobs = allJobs.filter((j) => j.feedback?.hidden);
  const badFitJobs = allJobs.filter((j) => j.feedback?.badFit);
  const whitelistedCompanies = (feedbackOverview?.companies ?? []).filter(
    (c) => c.status === 'whitelist',
  );
  const blacklistedCompanies = (feedbackOverview?.companies ?? []).filter(
    (c) => c.status === 'blacklist',
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

  const filteredSavedJobs = useMemo(() => filterJobsBySearch(savedJobs), [normalizedSearch, savedJobs]);
  const filteredHiddenJobs = useMemo(() => filterJobsBySearch(hiddenJobs), [normalizedSearch, hiddenJobs]);
  const filteredBadFitJobs = useMemo(() => filterJobsBySearch(badFitJobs), [normalizedSearch, badFitJobs]);

  function invalidateAfterAction() {
    void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
    if (profileId) {
      void queryClient.invalidateQueries({ queryKey: queryKeys.feedback.profile(profileId) });
    }
  }

  const unsaveMutation = useMutation({
    mutationFn: (jobId: string) => unsaveJob(profileId!, jobId),
    onSuccess: () => { invalidateAfterAction(); toast.success('Збережено скасовано'); },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const unhideMutation = useMutation({
    mutationFn: (jobId: string) => unhideJob(profileId!, jobId),
    onSuccess: () => { invalidateAfterAction(); toast.success('Вакансію показано знову'); },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const unmarkBadFitMutation = useMutation({
    mutationFn: (jobId: string) => unmarkJobBadFit(profileId!, jobId),
    onSuccess: () => { invalidateAfterAction(); toast.success('Позначку bad fit знято'); },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const removeWhitelistMutation = useMutation({
    mutationFn: (companyName: string) => removeCompanyWhitelist(profileId!, companyName),
    onSuccess: () => { invalidateAfterAction(); toast.success('Видалено зі списку дозволених'); },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const removeBlacklistMutation = useMutation({
    mutationFn: (companyName: string) => removeCompanyBlacklist(profileId!, companyName),
    onSuccess: () => { invalidateAfterAction(); toast.success('Видалено зі списку заблокованих'); },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const addWhitelistMutation = useMutation({
    mutationFn: (companyName: string) => addCompanyWhitelist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterAction();
      setWhitelistInput('');
      toast.success('Компанію додано до whitelist');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const addBlacklistMutation = useMutation({
    mutationFn: (companyName: string) => addCompanyBlacklist(profileId!, companyName),
    onSuccess: () => {
      invalidateAfterAction();
      setBlacklistInput('');
      toast.success('Компанію додано до blacklist');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
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
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
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
      <div className="jobDetails">
        <p className="emptyState">Create a profile to view feedback.</p>
      </div>
    );
  }

  const isLoading = jobsLoading || feedbackLoading;

  return (
    <div className="jobDetails">
      <PageHeader
        title="Feedback Center"
        description="Manage saved jobs, hidden roles, bad fits, and company preferences."
        breadcrumb={[
          { label: 'Dashboard', href: '/' },
          { label: 'Feedback' },
        ]}
      />

      {isLoading ? (
        <p className="emptyState">Loading feedback…</p>
      ) : (
        <>
          {summary && (
            <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
              <StatCard title="Saved" value={summary.savedJobsCount} icon={Bookmark} />
              <StatCard title="Hidden" value={summary.hiddenJobsCount} icon={EyeOff} />
              <StatCard title="Bad Fit" value={summary.badFitJobsCount} icon={ThumbsDown} />
              <StatCard title="Whitelisted" value={summary.whitelistedCompaniesCount} icon={ShieldCheck} />
              <StatCard title="Blacklisted" value={summary.blacklistedCompaniesCount} icon={ShieldOff} />
            </div>
          )}

          <div className="space-y-6">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center">
              <div className="flex w-full flex-wrap gap-2 sm:w-auto">
                {[
                  { id: 'saved', label: 'Saved', count: savedJobs.length, icon: Bookmark },
                  { id: 'hidden', label: 'Hidden', count: hiddenJobs.length, icon: EyeOff },
                  { id: 'bad-fit', label: 'Bad Fit', count: badFitJobs.length, icon: ThumbsDown },
                  { id: 'companies', label: 'Companies', count: whitelistedCompanies.length + blacklistedCompanies.length, icon: Building2 },
                ].map((tab) => (
                  <button
                    key={tab.id}
                    type="button"
                    onClick={() => setActiveTab(tab.id as FeedbackTab)}
                    className={cn(
                      'inline-flex items-center gap-2 rounded-lg border px-3 py-2 text-sm transition-colors',
                      activeTab === tab.id
                        ? 'border-primary/40 bg-primary/10 text-primary'
                        : 'border-border bg-surface-elevated/50 text-muted-foreground hover:bg-surface-hover hover:text-foreground',
                    )}
                  >
                    <tab.icon className="h-4 w-4" />
                    <span>{tab.label}</span>
                    <span className="rounded-md bg-black/20 px-1.5 py-0.5 text-[11px] leading-none">
                      {tab.count}
                    </span>
                  </button>
                ))}
              </div>

              {activeTab !== 'companies' && (
                <div className="relative w-full sm:ml-auto sm:max-w-sm">
                  <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <input
                    type="search"
                    value={searchQuery}
                    onChange={(event) => setSearchQuery(event.target.value)}
                    placeholder="Search jobs..."
                    className="w-full pl-9"
                  />
                </div>
              )}
            </div>

            {activeTab === 'saved' && (
              <Section title="Saved Jobs" icon={<Bookmark size={16} />} count={filteredSavedJobs.length}>
                {filteredSavedJobs.length === 0 ? (
                  <p className="emptyState">
                    {searchQuery ? 'No saved jobs match this query.' : 'No saved jobs.'}
                  </p>
                ) : (
                  <div className="flex flex-col gap-3">
                    {filteredSavedJobs.map((job) => (
                      <JobRow
                        key={job.id}
                        job={job}
                        actionLabel="Unsave"
                        onAction={(id) => unsaveMutation.mutate(id)}
                        isPending={unsaveMutation.isPending}
                      />
                    ))}
                  </div>
                )}
              </Section>
            )}

            {activeTab === 'hidden' && (
              <Section title="Hidden Jobs" icon={<EyeOff size={16} />} count={filteredHiddenJobs.length}>
                <p className="mb-4 text-sm text-muted-foreground">
                  Hidden jobs stay out of the main feed until you restore them.
                </p>
                {filteredHiddenJobs.length === 0 ? (
                  <p className="emptyState">
                    {searchQuery ? 'No hidden jobs match this query.' : 'No hidden jobs.'}
                  </p>
                ) : (
                  <div className="flex flex-col gap-3">
                    {filteredHiddenJobs.map((job) => (
                      <JobRow
                        key={job.id}
                        job={job}
                        actionLabel="Unhide"
                        onAction={(id) => unhideMutation.mutate(id)}
                        isPending={unhideMutation.isPending}
                      />
                    ))}
                  </div>
                )}
              </Section>
            )}

            {activeTab === 'bad-fit' && (
              <Section title="Bad Fit" icon={<ThumbsDown size={16} />} count={filteredBadFitJobs.length}>
                <p className="mb-4 text-sm text-muted-foreground">
                  Jobs marked as bad fit influence future ranking and recommendations.
                </p>
                {filteredBadFitJobs.length === 0 ? (
                  <p className="emptyState">
                    {searchQuery ? 'No bad-fit jobs match this query.' : 'No jobs marked as bad fit.'}
                  </p>
                ) : (
                  <div className="flex flex-col gap-3">
                    {filteredBadFitJobs.map((job) => (
                      <JobRow
                        key={job.id}
                        job={job}
                        actionLabel="Unmark"
                        onAction={(id) => unmarkBadFitMutation.mutate(id)}
                        isPending={unmarkBadFitMutation.isPending}
                      />
                    ))}
                  </div>
                )}
              </Section>
            )}

            {activeTab === 'companies' && (
              <div className="grid gap-6 lg:grid-cols-2">
                <Section
                  title="Whitelisted Companies"
                  icon={<Star size={16} />}
                  count={whitelistedCompanies.length}
                >
                  <p className="mb-4 text-sm text-muted-foreground">
                    Jobs from these companies should be prioritized in the feed.
                  </p>
                  <div className="mb-4 flex gap-2">
                    <input
                      type="text"
                      value={whitelistInput}
                      onChange={(event) => setWhitelistInput(event.target.value)}
                      onKeyDown={(event) => {
                        if (event.key === 'Enter') submitCompany('whitelist');
                      }}
                      placeholder="Add company..."
                      className="flex-1"
                    />
                    <Button
                      type="button"
                      variant="outline"
                      size="icon"
                      onClick={() => submitCompany('whitelist')}
                      disabled={addWhitelistMutation.isPending || !whitelistInput.trim()}
                    >
                      <Plus className="h-4 w-4" />
                    </Button>
                  </div>
                  {whitelistedCompanies.length === 0 ? (
                    <p className="emptyState">No whitelisted companies.</p>
                  ) : (
                    <div className="flex flex-col gap-3">
                      {whitelistedCompanies.map((company) => (
                        <Card
                          key={company.normalizedCompanyName}
                          className="border-fit-excellent/20 bg-fit-excellent/5 px-4 py-4"
                        >
                          <div className="flex items-start justify-between gap-4">
                            <div className="flex items-start gap-3">
                              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-fit-excellent/10 text-fit-excellent">
                                <Building2 className="h-4 w-4" />
                              </div>
                              <div className="min-w-0">
                                <p className="m-0 text-sm font-semibold text-foreground">{company.companyName}</p>
                                <p className="mt-1 mb-0 text-xs text-muted-foreground">
                                  Prioritized for future ranking.
                                </p>
                              </div>
                            </div>
                            <div className="flex items-center gap-2">
                              <Button
                                type="button"
                                variant="ghost"
                                size="icon"
                                className="h-8 w-8 text-muted-foreground hover:text-destructive"
                                onClick={() =>
                                  moveCompanyMutation.mutate({
                                    companyName: company.companyName,
                                    nextStatus: 'blacklist',
                                  })
                                }
                                disabled={moveCompanyMutation.isPending}
                                title="Move to blacklist"
                              >
                                <Ban className="h-4 w-4" />
                              </Button>
                              <Button
                                type="button"
                                variant="ghost"
                                size="sm"
                                onClick={() => removeWhitelistMutation.mutate(company.companyName)}
                                disabled={removeWhitelistMutation.isPending}
                              >
                                Remove
                              </Button>
                            </div>
                          </div>
                        </Card>
                      ))}
                    </div>
                  )}
                </Section>

                <Section
                  title="Blacklisted Companies"
                  icon={<Ban size={16} />}
                  count={blacklistedCompanies.length}
                >
                  <p className="mb-4 text-sm text-muted-foreground">
                    Jobs from these companies should be hidden from the main feed.
                  </p>
                  <div className="mb-4 flex gap-2">
                    <input
                      type="text"
                      value={blacklistInput}
                      onChange={(event) => setBlacklistInput(event.target.value)}
                      onKeyDown={(event) => {
                        if (event.key === 'Enter') submitCompany('blacklist');
                      }}
                      placeholder="Add company..."
                      className="flex-1"
                    />
                    <Button
                      type="button"
                      variant="outline"
                      size="icon"
                      onClick={() => submitCompany('blacklist')}
                      disabled={addBlacklistMutation.isPending || !blacklistInput.trim()}
                    >
                      <Plus className="h-4 w-4" />
                    </Button>
                  </div>
                  {blacklistedCompanies.length === 0 ? (
                    <p className="emptyState">No blacklisted companies.</p>
                  ) : (
                    <div className="flex flex-col gap-3">
                      {blacklistedCompanies.map((company) => (
                        <Card
                          key={company.normalizedCompanyName}
                          className="border-destructive/20 bg-destructive/5 px-4 py-4"
                        >
                          <div className="flex items-start justify-between gap-4">
                            <div className="flex items-start gap-3">
                              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-destructive/10 text-destructive">
                                <Building2 className="h-4 w-4" />
                              </div>
                              <div className="min-w-0">
                                <p className="m-0 text-sm font-semibold text-foreground">{company.companyName}</p>
                                <p className="mt-1 mb-0 text-xs text-muted-foreground">
                                  Hidden from future ranking.
                                </p>
                              </div>
                            </div>
                            <div className="flex items-center gap-2">
                              <Button
                                type="button"
                                variant="ghost"
                                size="icon"
                                className="h-8 w-8 text-muted-foreground hover:text-fit-excellent"
                                onClick={() =>
                                  moveCompanyMutation.mutate({
                                    companyName: company.companyName,
                                    nextStatus: 'whitelist',
                                  })
                                }
                                disabled={moveCompanyMutation.isPending}
                                title="Move to whitelist"
                              >
                                <Star className="h-4 w-4" />
                              </Button>
                              <Button
                                type="button"
                                variant="ghost"
                                size="sm"
                                onClick={() => removeBlacklistMutation.mutate(company.companyName)}
                                disabled={removeBlacklistMutation.isPending}
                              >
                                Remove
                              </Button>
                            </div>
                          </div>
                        </Card>
                      ))}
                    </div>
                  )}
                </Section>
              </div>
            )}
          </div>
        </>
      )}
    </div>
  );
}
