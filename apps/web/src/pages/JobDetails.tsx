import { useEffect, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import {
  AlertCircle,
  BarChart3,
  Bookmark,
  BookmarkCheck,
  Briefcase,
  Building2,
  CalendarClock,
  Check,
  CheckCircle2,
  Copy,
  EyeOff,
  ExternalLink,
  MapPin,
  Sparkles,
  Target,
} from 'lucide-react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type { Application } from '@job-copilot/shared';

import {
  addCompanyBlacklist,
  addCompanyWhitelist,
  hideJobForProfile,
  markJobBadFit,
  markJobSaved,
  removeCompanyBlacklist,
  removeCompanyWhitelist,
  unhideJob,
  unmarkJobBadFit,
  unsaveJob,
} from '../api/feedback';
import type { FitAnalysis } from '../api/jobs';
import { analyzeFit, getJob } from '../api/jobs';
import { logUserEvent } from '../api/events';
import {
  getCoverLetterDraft,
  getInterviewPrep,
  getJobFitExplanation,
  type JobFitExplanation,
  type CoverLetterDraft,
  type InterviewPrep,
} from '../api/enrichment';
import { createApplication, getApplications } from '../api/applications';
import { SkeletonPage } from '../components/Skeleton';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { EmptyState } from '../components/ui/EmptyState';
import { FitScoreBox } from '../components/ui/FitScoreBox';
import { Page, PageGrid } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { StatusBadge } from '../components/ui/StatusBadge';
import { cn } from '../lib/cn';
import {
  invalidateApplicationSummaryQueries,
  invalidateFeedbackViewQueries,
  invalidateJobAiQueries,
} from '../lib/queryInvalidation';
import { queryKeys } from '../queryKeys';
import { FeedbackButton, HeroMetric, Section, formatDate, formatSalary } from './job-details/components';
import { JobDetailsAiTab } from './job-details/JobDetailsAiTab';
import { JobDetailsMatchTab } from './job-details/JobDetailsMatchTab';

function readProfileId() {
  return window.localStorage.getItem('engine_api_profile_id');
}

export default function JobDetails() {
  const { id } = useParams<{ id: string }>();
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = useState<'overview' | 'match' | 'ai' | 'lifecycle'>('overview');
  const [copied, setCopied] = useState(false);
  const [generateCoverLetter, setGenerateCoverLetter] = useState(false);
  const [generateInterviewPrep, setGenerateInterviewPrep] = useState(false);

  const {
    data: job,
    isLoading,
    error,
  } = useQuery({
    queryKey: queryKeys.jobs.detail(id!, readProfileId()),
    queryFn: () => getJob(id!),
    enabled: !!id,
  });

  const { data: applications = [] } = useQuery<Application[]>({
    queryKey: queryKeys.applications.all(),
    queryFn: getApplications,
  });

  const profileId = readProfileId();

  useEffect(() => {
    if (!profileId || !job?.id) return;

    void logUserEvent(profileId, {
      eventType: 'job_opened',
      jobId: job.id,
      payloadJson: { surface: 'job_details' },
    }).catch(() => null);
  }, [job?.id, profileId]);

  const { data: fit } = useQuery<FitAnalysis>({
    queryKey: queryKeys.ml.fit(profileId ?? '', id!),
    queryFn: () => analyzeFit(profileId!, id!),
    enabled: !!profileId && !!id,
    staleTime: 2 * 60_000,
    retry: false,
  });

  const deterministicFit = fit && job
    ? {
        jobId: fit.jobId,
        score: fit.score,
        scoreBreakdown: fit.scoreBreakdown,
        matchedRoles: fit.matchedRoles,
        matchedSkills: fit.matchedSkills,
        matchedKeywords: fit.matchedKeywords,
        missingSignals: fit.missingTerms,
        sourceMatch: false,
        workModeMatch: undefined,
        regionMatch: undefined,
        descriptionQuality: fit.descriptionQuality,
        positiveReasons: fit.positiveReasons,
        negativeReasons: fit.negativeReasons,
        reasons: [...fit.positiveReasons, ...fit.negativeReasons],
      }
    : null;

  const { data: fitExplanation, isLoading: fitExplanationLoading } = useQuery<JobFitExplanation>({
    queryKey: queryKeys.ml.fitExplanation(profileId ?? '', id ?? ''),
    queryFn: () =>
      getJobFitExplanation({
        profileId: profileId!,
        analyzedProfile: null,
        searchProfile: null,
        rankedJob: job!,
        deterministicFit: deterministicFit!,
      }),
    enabled: activeTab === 'ai' && !!profileId && !!deterministicFit,
    staleTime: 10 * 60_000,
    retry: false,
  });

  const { data: coverLetter, isLoading: coverLetterLoading } = useQuery<CoverLetterDraft>({
    queryKey: queryKeys.ml.coverLetter(profileId ?? '', id ?? ''),
    queryFn: () =>
      getCoverLetterDraft({
        profileId: profileId!,
        analyzedProfile: null,
        searchProfile: null,
        rankedJob: job!,
        deterministicFit: deterministicFit!,
        jobFitExplanation: fitExplanation ?? null,
      }),
    enabled: generateCoverLetter && !!profileId && !!deterministicFit,
    staleTime: 30 * 60_000,
    retry: false,
  });

  const { data: interviewPrep, isLoading: interviewPrepLoading } = useQuery<InterviewPrep>({
    queryKey: queryKeys.ml.interviewPrep(profileId ?? '', id ?? ''),
    queryFn: () =>
      getInterviewPrep({
        profileId: profileId!,
        analyzedProfile: null,
        searchProfile: null,
        rankedJob: job!,
        deterministicFit: deterministicFit!,
        jobFitExplanation: fitExplanation ?? null,
      }),
    enabled: generateInterviewPrep && !!profileId && !!deterministicFit,
    staleTime: 30 * 60_000,
    retry: false,
  });

  const existing = applications.find((application) => application.jobId === id);
  const isSaved = job?.feedback?.saved || !!existing;
  const isHidden = job?.feedback?.hidden;
  const isBadFit = job?.feedback?.badFit;
  const companyStatus = job?.feedback?.companyStatus;

  const invalidateFeedbackQueries = () => {
    void invalidateFeedbackViewQueries(queryClient, profileId);
    void invalidateJobAiQueries(queryClient, profileId, id);
  };

  const saveMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) throw new Error('Create a profile first');
      await markJobSaved(profileId, id!);
      if (!existing) await createApplication({ jobId: id!, status: 'saved' });
    },
    onSuccess: () => {
      invalidateFeedbackQueries();
      void invalidateApplicationSummaryQueries(queryClient);
      toast.success('Збережено в pipeline');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const unsaveMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) throw new Error('Create a profile first');
      await unsaveJob(profileId, id!);
    },
    onSuccess: () => {
      invalidateFeedbackQueries();
      toast.success('Знято з обраного');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const hideMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) throw new Error('Create a profile first');
      await hideJobForProfile(profileId, id!);
    },
    onSuccess: () => {
      invalidateFeedbackQueries();
      toast.success('Вакансію приховано');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const unhideMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) throw new Error('Create a profile first');
      await unhideJob(profileId, id!);
    },
    onSuccess: () => {
      invalidateFeedbackQueries();
      toast.success('Вакансію показано');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const badFitMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) throw new Error('Create a profile first');
      await markJobBadFit(profileId, id!);
    },
    onSuccess: () => {
      invalidateFeedbackQueries();
      toast.success('Позначено як bad fit');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const unmarkBadFitMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) throw new Error('Create a profile first');
      await unmarkJobBadFit(profileId, id!);
    },
    onSuccess: () => {
      invalidateFeedbackQueries();
      toast.success('Позначку bad fit знято');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const companyFeedbackMutation = useMutation({
    mutationFn: async (nextStatus: 'whitelist' | 'blacklist') => {
      if (!profileId) throw new Error('Create a profile first');

      if (nextStatus === 'whitelist') {
        if (companyStatus === 'whitelist') await removeCompanyWhitelist(profileId, job!.company);
        else await addCompanyWhitelist(profileId, job!.company);
        return;
      }

      if (companyStatus === 'blacklist') await removeCompanyBlacklist(profileId, job!.company);
      else await addCompanyBlacklist(profileId, job!.company);
    },
    onSuccess: () => {
      invalidateFeedbackQueries();
      toast.success('Оновлено список компанії');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  if (isLoading) return <SkeletonPage />;
  if (!job) {
    return (
      <Page>
        <EmptyState message={error instanceof Error ? error.message : 'Вакансія не знайдена'} />
      </Page>
    );
  }

  const salary = formatSalary(job.salaryMin, job.salaryMax, job.salaryCurrency);
  const sourceLabel = job.primaryVariant?.source
    ? job.primaryVariant.source.replace('_', '.')
    : 'Unknown source';
  const descriptionQuality = fit?.descriptionQuality ?? job.presentation?.descriptionQuality;

  const topBadges = [sourceLabel, job.seniority ?? null, job.remoteType ?? null].filter(
    Boolean,
  ) as string[];

  const skillBadges = [...(job.presentation?.badges ?? []), ...(fit?.matchedTerms ?? [])].slice(
    0,
    10,
  );

  const handleCopy = async () => {
    if (typeof window === 'undefined' || !navigator.clipboard) return;

    await navigator.clipboard.writeText(window.location.href);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Page>
      <PageHeader
        title={job.title}
        description={job.company}
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Jobs' }, { label: job.company }]}
        actions={
          <>
            {existing ? (
              <Link to={`/applications/${existing.id}`} className="no-underline">
                <Button variant="outline" size="sm" className="bg-primary/10 border-primary/30 text-primary">
                  <BookmarkCheck className="h-4 w-4" />
                  {existing.status}
                </Button>
              </Link>
            ) : isSaved ? (
              <Button
                variant="outline"
                size="sm"
                onClick={() => unsaveMutation.mutate()}
                disabled={unsaveMutation.isPending}
                className="bg-primary/10 border-primary/30 text-primary"
              >
                <BookmarkCheck className="h-4 w-4 text-primary" />
                {unsaveMutation.isPending ? 'Знімаємо…' : 'Saved'}
              </Button>
            ) : (
              <Button onClick={() => saveMutation.mutate()} disabled={saveMutation.isPending}>
                <Bookmark className="h-4 w-4" />
                {saveMutation.isPending ? 'Зберігаємо…' : 'Save'}
              </Button>
            )}
            <Button variant="outline" size="sm" onClick={() => void handleCopy()}>
              {copied ? (
                <Check className="h-4 w-4 text-fit-excellent" />
              ) : (
                <Copy className="h-4 w-4" />
              )}
              {copied ? 'Copied' : 'Share'}
            </Button>
          </>
        }
      />

      <div className="overflow-hidden rounded-[var(--radius-card)] border border-border bg-card">
        <div className="relative">
          <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-primary/8 via-accent/6 to-transparent" />
          <div className="relative flex flex-col gap-6 p-7 lg:flex-row lg:items-start lg:justify-between">
            <div className="flex min-w-0 gap-4">
              <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-2xl border border-primary/20 bg-primary/10 text-primary">
                <Building2 className="h-8 w-8" />
              </div>
              <div className="min-w-0 space-y-4">
                <div className="flex flex-wrap gap-2">
                  {topBadges.map((badge) => (
                    <Badge key={badge} variant="muted" className="px-2.5 py-1 text-xs">
                      {badge}
                    </Badge>
                  ))}
                  <StatusBadge
                    status={
                      job.lifecycleStage ?? (job.isActive === false ? 'inactive' : 'active')
                    }
                  />
                  {isBadFit ? <StatusBadge status="bad fit" /> : null}
                  {companyStatus === 'blacklist' ? (
                    <StatusBadge status="blacklist" label="company blacklisted" />
                  ) : null}
                  {companyStatus === 'whitelist' ? (
                    <StatusBadge status="whitelist" label="company whitelisted" />
                  ) : null}
                  {descriptionQuality === 'weak' ? (
                    <StatusBadge status="bad fit" label="description may be incomplete" />
                  ) : null}
                </div>

                <div>
                  <h2 className="m-0 text-2xl font-bold text-card-foreground">{job.title}</h2>
                  <p className="m-0 mt-2 text-base text-muted-foreground">{job.company}</p>
                </div>

                <div className="flex flex-wrap gap-3 text-sm text-muted-foreground">
                  {salary ? (
                    <span className="inline-flex items-center gap-1.5 rounded-full border border-border bg-white/[0.05] px-3 py-1.5">
                      <MapPin className="h-4 w-4" />
                      {salary}
                    </span>
                  ) : null}
                  {job.postedAt ? (
                    <span className="inline-flex items-center gap-1.5 rounded-full border border-border bg-white/[0.05] px-3 py-1.5">
                      <CalendarClock className="h-4 w-4" />
                      Posted {formatDate(job.postedAt)}
                    </span>
                  ) : null}
                  <span className="inline-flex items-center gap-1.5 rounded-full border border-border bg-white/[0.05] px-3 py-1.5">
                    <MapPin className="h-4 w-4" />
                    {sourceLabel}
                  </span>
                </div>
              </div>
            </div>

            <div className="grid gap-3 sm:grid-cols-3 lg:min-w-[420px]">
              <HeroMetric
                label="Fit score"
                value={fit ? `${fit.score}%` : 'Pending'}
                icon={Target}
              />
              <HeroMetric
                label="Matched terms"
                value={fit?.matchedTerms.length ?? 0}
                icon={Sparkles}
              />
              <HeroMetric
                label="Pipeline"
                value={existing ? existing.status : 'Not saved'}
                icon={BookmarkCheck}
              />
            </div>
          </div>
        </div>
      </div>

      <PageGrid
        aside={
          <div className="space-y-6">
            {profileId ? (
              <Section
                title="Fit Snapshot"
                description="Compact view of the current match score and the strongest signals."
                icon={Sparkles}
              >
                {fit ? (
                  <div className="space-y-5">
                    <FitScoreBox score={fit.score} size="lg" showLabel className="mx-auto" />
                    {fit.evidence.length > 0 ? (
                      <div className="space-y-3">
                        {fit.evidence.slice(0, 3).map((entry) => (
                          <div
                            key={entry}
                            className="flex items-start gap-3 rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-3"
                          >
                            <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0 text-fit-excellent" />
                            <p className="m-0 text-sm leading-6 text-card-foreground">{entry}</p>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <EmptyState
                        message="No evidence returned yet."
                        className="px-4 py-4 text-left"
                      />
                    )}
                  </div>
                ) : (
                  <p className="m-0 text-sm text-muted-foreground">Аналізуємо…</p>
                )}
              </Section>
            ) : null}

            <Section
              title="Feedback Actions"
              description="Use explicit controls to change ranking behavior for this role and company."
              icon={BarChart3}
            >
              <div className="space-y-3">
                {isHidden ? (
                  <FeedbackButton
                    disabled={unhideMutation.isPending}
                    onClick={() => unhideMutation.mutate()}
                  >
                    <EyeOff className="h-4 w-4" />
                    {unhideMutation.isPending ? 'Показуємо…' : 'Unhide job'}
                  </FeedbackButton>
                ) : (
                  <FeedbackButton
                    disabled={hideMutation.isPending}
                    onClick={() => hideMutation.mutate()}
                  >
                    <EyeOff className="h-4 w-4" />
                    {hideMutation.isPending ? 'Ховаємо…' : 'Hide job'}
                  </FeedbackButton>
                )}

                {isBadFit ? (
                  <FeedbackButton
                    disabled={unmarkBadFitMutation.isPending}
                    onClick={() => unmarkBadFitMutation.mutate()}
                  >
                    <AlertCircle className="h-4 w-4" />
                    {unmarkBadFitMutation.isPending ? 'Знімаємо…' : 'Remove bad fit'}
                  </FeedbackButton>
                ) : (
                  <FeedbackButton
                    className="text-destructive hover:text-destructive"
                    disabled={badFitMutation.isPending}
                    onClick={() => badFitMutation.mutate()}
                  >
                    <AlertCircle className="h-4 w-4" />
                    {badFitMutation.isPending ? 'Позначаємо…' : 'Not a good fit'}
                  </FeedbackButton>
                )}

                <FeedbackButton
                  className={cn(companyStatus === 'whitelist' && 'bg-primary/10 border-primary/30')}
                  disabled={companyFeedbackMutation.isPending}
                  onClick={() => companyFeedbackMutation.mutate('whitelist')}
                >
                  <CheckCircle2 className="h-4 w-4" />
                  {companyStatus === 'whitelist' ? 'Unwhitelist company' : 'Whitelist company'}
                </FeedbackButton>

                <FeedbackButton
                  className={cn(
                    companyStatus === 'blacklist' &&
                      'bg-destructive/10 border-destructive/30 text-destructive',
                  )}
                  disabled={companyFeedbackMutation.isPending}
                  onClick={() => companyFeedbackMutation.mutate('blacklist')}
                >
                  <AlertCircle className="h-4 w-4" />
                  {companyStatus === 'blacklist' ? 'Unblacklist company' : 'Blacklist company'}
                </FeedbackButton>
              </div>
            </Section>

            <Section
              title="Source Actions"
              description="Open the original posting or jump into the application record when available."
              icon={ExternalLink}
            >
              <div className="space-y-2">
                {job.primaryVariant?.sourceUrl ? (
                  <a
                    href={job.primaryVariant.sourceUrl}
                    target="_blank"
                    rel="noreferrer"
                    className="block no-underline"
                  >
                    <Button className="w-full">
                      <ExternalLink className="h-4 w-4" />
                      Apply on source
                    </Button>
                  </a>
                ) : null}
                {existing ? (
                  <Link to={`/applications/${existing.id}`} className="block no-underline">
                    <Button variant="outline" className="w-full">
                      <BookmarkCheck className="h-4 w-4" />
                      Open application record
                    </Button>
                  </Link>
                ) : null}
              </div>
            </Section>
          </div>
        }
      >
        <div className="space-y-6">
          <div className="flex flex-wrap gap-2">
            {[
              { id: 'overview', label: 'Overview' },
              { id: 'match', label: 'Match' },
              { id: 'ai', label: 'AI Analysis' },
              { id: 'lifecycle', label: 'Lifecycle' },
            ].map((tab) => (
              <button
                key={tab.id}
                type="button"
                onClick={() => setActiveTab(tab.id as typeof activeTab)}
                className={cn(
                  'inline-flex items-center rounded-full border px-3 py-2 text-sm transition-colors',
                  activeTab === tab.id
                    ? 'border-primary bg-primary text-primary-foreground'
                    : 'border-border bg-surface-elevated/50 text-muted-foreground hover:bg-surface-hover hover:text-foreground',
                )}
              >
                {tab.label}
              </button>
            ))}
          </div>

          {activeTab === 'overview' ? (
            <Section
              title="Role Overview"
              description="Read the job description and the strongest structured signals extracted from the posting."
              icon={Briefcase}
            >
              <div className="space-y-5">
                <div className="rounded-2xl border border-border/70 bg-white/[0.03] p-4">
                  <p className="m-0 text-sm font-semibold text-card-foreground">Job description</p>
                  <div className="mt-4 whitespace-pre-wrap text-sm leading-7 text-muted-foreground">
                    {job.description || 'No description available.'}
                  </div>
                </div>

                {descriptionQuality === 'weak' ? (
                  <div className="rounded-2xl border border-content-warning/40 bg-content-warning/10 p-4">
                    <p className="m-0 text-sm font-semibold text-card-foreground">
                      Description quality warning
                    </p>
                    <p className="m-0 mt-3 text-sm leading-7 text-muted-foreground">
                      This vacancy looks partially extracted or too short. Treat the fit score as
                      lower-confidence until the source page or a richer reparse confirms the
                      missing context.
                    </p>
                  </div>
                ) : null}

                {skillBadges.length > 0 ? (
                  <div className="rounded-2xl border border-border/70 bg-white/[0.03] p-4">
                    <p className="m-0 text-sm font-semibold text-card-foreground">
                      Skills and signals
                    </p>
                    <div className="mt-4 flex flex-wrap gap-2">
                      {skillBadges.map((item) => (
                        <Badge key={item} variant="muted" className="px-3 py-1 text-xs">
                          {item}
                        </Badge>
                      ))}
                    </div>
                  </div>
                ) : null}
              </div>
            </Section>
          ) : null}

          {activeTab === 'match' ? <JobDetailsMatchTab fit={fit} /> : null}

          {activeTab === 'ai' ? (
            <JobDetailsAiTab
              profileId={profileId}
              deterministicFit={deterministicFit}
              fitExplanation={fitExplanation}
              fitExplanationLoading={fitExplanationLoading}
              coverLetter={coverLetter}
              coverLetterLoading={coverLetterLoading}
              interviewPrep={interviewPrep}
              interviewPrepLoading={interviewPrepLoading}
              setGenerateCoverLetter={setGenerateCoverLetter}
              setGenerateInterviewPrep={setGenerateInterviewPrep}
            />
          ) : null}

          {activeTab === 'lifecycle' ? (
            <Section
              title="Lifecycle Metadata"
              description="Timeline and source-specific identifiers from the canonical job record."
              icon={CalendarClock}
            >
              <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                <HeroMetric
                  label="First seen"
                  value={formatDate(job.firstSeenAt) ?? 'n/a'}
                  icon={CalendarClock}
                />
                <HeroMetric
                  label="Last seen"
                  value={formatDate(job.lastSeenAt) ?? 'n/a'}
                  icon={CalendarClock}
                />
                <HeroMetric
                  label="Inactive at"
                  value={formatDate(job.inactivatedAt) ?? 'n/a'}
                  icon={CalendarClock}
                />
                <HeroMetric
                  label="Reactivated"
                  value={formatDate(job.reactivatedAt) ?? 'n/a'}
                  icon={CalendarClock}
                />
                <HeroMetric
                  label="Source id"
                  value={job.primaryVariant?.sourceJobId ?? 'n/a'}
                  icon={MapPin}
                />
              </div>
            </Section>
          ) : null}
        </div>
      </PageGrid>
    </Page>
  );
}
