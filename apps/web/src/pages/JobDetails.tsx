import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import {
  AlertCircle,
  Check,
  Bookmark,
  BookmarkCheck,
  Briefcase,
  Building2,
  CheckCircle2,
  Copy,
  EyeOff,
  ExternalLink,
  MapPin,
  Sparkles,
} from 'lucide-react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type { Application } from '@job-copilot/shared';

import {
  type FitAnalysis,
  addCompanyBlacklist,
  addCompanyWhitelist,
  analyzeFit,
  createApplication,
  getApplications,
  getJob,
  hideJobForProfile,
  logUserEvent,
  markJobBadFit,
  markJobSaved,
  removeCompanyBlacklist,
  removeCompanyWhitelist,
  unhideJob,
  unmarkJobBadFit,
  unsaveJob,
} from '../api';
import { queryKeys } from '../queryKeys';
import { SkeletonPage } from '../components/Skeleton';
import { Button } from '../components/ui/Button';
import { Badge } from '../components/ui/Badge';
import { Card, CardContent, CardHeader } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { FitScoreBox, FitScoreCircular } from '../components/ui/FitScoreBox';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { StatusBadge } from '../components/ui/StatusBadge';
import { cn } from '../lib/cn';

function readProfileId() {
  return window.localStorage.getItem('engine_api_profile_id');
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function formatSalary(min?: number, max?: number, currency?: string) {
  if (!min && !max) return null;
  const sym = currency === 'USD' ? '$' : currency === 'EUR' ? '€' : (currency ?? '');
  const fmt = (n: number) => `${sym}${n.toLocaleString()}`;
  if (min && max) return `${fmt(min)} – ${fmt(max)}`;
  return min ? `від ${fmt(min)}` : `до ${fmt(max!)}`;
}

function formatDate(value?: string) {
  if (!value) return null;
  const d = new Date(value);
  return Number.isNaN(d.getTime())
    ? null
    : d.toLocaleDateString('uk-UA', { day: 'numeric', month: 'short', year: 'numeric' });
}

// ── Page ──────────────────────────────────────────────────────────────────────

export default function JobDetails() {
  const { id } = useParams<{ id: string }>();
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = useState<'overview' | 'match' | 'lifecycle'>('overview');
  const [copied, setCopied] = useState(false);

  const { data: job, isLoading, error } = useQuery({
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
    staleTime: 10 * 60_000,
    retry: false,
  });

  const existing = applications.find((a) => a.jobId === id);
  const isSaved = job?.feedback?.saved || !!existing;
  const isHidden = job?.feedback?.hidden;
  const isBadFit = job?.feedback?.badFit;
  const companyStatus = job?.feedback?.companyStatus;

  const saveMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) throw new Error('Create a profile first');
      await markJobSaved(profileId, id!);
      if (!existing) await createApplication({ jobId: id!, status: 'saved' });
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.applications.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.stats() });
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
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.detail(id!, profileId) });
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
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.detail(id!, profileId) });
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
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.detail(id!, profileId) });
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
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.detail(id!, profileId) });
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
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.detail(id!, profileId) });
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
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.detail(id!, profileId) });
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

  const topBadges = [
    job.primaryVariant?.source ? job.primaryVariant.source.replace('_', '.') : null,
    job.seniority ?? null,
    job.remoteType ?? null,
  ].filter(Boolean) as string[];

  const skillBadges = [
    ...(job.presentation?.badges ?? []),
    ...(fit?.matchedTerms ?? []),
  ].slice(0, 10);

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
        breadcrumb={[
          { label: 'Dashboard', href: '/' },
          { label: 'Jobs' },
          { label: job.company },
        ]}
        actions={(
          <>
            {existing ? (
              <div className="inline-flex items-center gap-1.5 rounded-full border border-primary/25 bg-primary/15 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-primary">
                <BookmarkCheck size={13} /> {existing.status}
              </div>
            ) : isSaved ? (
              <Button
                variant="outline"
                size="sm"
                onClick={() => unsaveMutation.mutate()}
                disabled={unsaveMutation.isPending}
                className="bg-primary/10 border-primary/30"
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
            <Button
              variant="outline"
              size="sm"
              onClick={() => (isHidden ? unhideMutation.mutate() : hideMutation.mutate())}
              disabled={hideMutation.isPending || unhideMutation.isPending}
            >
              <EyeOff className="h-4 w-4" />
              {isHidden ? 'Unhide' : 'Hide'}
            </Button>
            <Button variant="outline" size="sm" onClick={() => void handleCopy()}>
              {copied ? <Check className="h-4 w-4 text-fit-excellent" /> : <Copy className="h-4 w-4" />}
              {copied ? 'Copied' : 'Share'}
            </Button>
          </>
        )}
      />

      {/* Main grid: 2/3 + 1/3 */}
      <div className="grid lg:grid-cols-3 gap-6">

        {/* ── Main column ── */}
        <div className="lg:col-span-2 space-y-4">

          {/* Job header card */}
          <Card className="border-border bg-card overflow-hidden">
            <CardContent className="p-0">
              <div className="relative">
                <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-primary/5 via-transparent to-accent/5" />
                <div className="relative p-6">
                  <div className="flex flex-col gap-6 md:flex-row md:items-start">
                    <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-xl border border-primary/20 bg-primary/10">
                      <Building2 className="h-8 w-8 text-primary" />
                    </div>

                    <div className="min-w-0 flex-1">
                      <div className="mb-2 flex flex-wrap items-center gap-2">
                        {topBadges.map((badge) => (
                          <Badge key={badge} variant="muted" className="px-2.5 py-1 text-xs">
                            {badge}
                          </Badge>
                        ))}
                        <StatusBadge
                          status={job.lifecycleStage ?? (job.isActive === false ? 'inactive' : 'active')}
                        />
                        {isBadFit && <StatusBadge status="bad fit" />}
                        {companyStatus === 'blacklist' && (
                          <StatusBadge status="blacklist" label="company blacklisted" />
                        )}
                        {companyStatus === 'whitelist' && (
                          <StatusBadge status="whitelist" label="company whitelisted" />
                        )}
                      </div>

                      <h2 className="mb-1 text-xl font-bold text-card-foreground">{job.title}</h2>
                      <p className="mb-4 text-lg text-muted-foreground">{job.company}</p>

                      <div className="flex flex-wrap items-center gap-4 text-sm text-muted-foreground">
                        {job.primaryVariant?.source && (
                          <span className="flex items-center gap-1.5">
                            <MapPin className="h-4 w-4" />
                            {job.primaryVariant.source.replace('_', '.')}
                          </span>
                        )}
                        {salary && (
                          <span className="flex items-center gap-1.5">
                            <Briefcase className="h-4 w-4" />
                            {salary}
                          </span>
                        )}
                        {job.postedAt && (
                          <span className="text-sm text-muted-foreground">
                            Posted {formatDate(job.postedAt)}
                          </span>
                        )}
                      </div>
                    </div>

                    {fit && (
                      <div className="flex shrink-0 flex-col items-center">
                        <FitScoreCircular score={fit.score} size="lg" showLabel />
                      </div>
                    )}
                  </div>

                  <div className="mt-6 flex flex-wrap gap-2 border-t border-border pt-6">
                    {job.primaryVariant?.sourceUrl && (
                      <a
                        href={job.primaryVariant.sourceUrl}
                        target="_blank"
                        rel="noreferrer"
                        className="inline-flex items-center gap-2 rounded-xl bg-[image:var(--gradient-button)] px-4 py-2.5 text-sm font-medium text-white no-underline shadow-[0_18px_40px_rgba(0,0,0,0.24)]"
                      >
                        <ExternalLink className="h-4 w-4" />
                        Apply on source
                      </a>
                    )}
                    {profileId && (
                      <Button
                        variant="outline"
                        className="flex-1 sm:flex-none"
                        onClick={() => setActiveTab('match')}
                      >
                        <Sparkles className="h-4 w-4" />
                        AI fit details
                      </Button>
                    )}
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          <div className="flex flex-wrap gap-2">
            {[
              { id: 'overview', label: 'Overview' },
              { id: 'match', label: 'Match' },
              { id: 'lifecycle', label: 'Lifecycle' },
            ].map((tab) => (
              <button
                key={tab.id}
                type="button"
                onClick={() => setActiveTab(tab.id as typeof activeTab)}
                className={cn(
                  'inline-flex items-center rounded-lg border px-3 py-2 text-sm transition-colors',
                  activeTab === tab.id
                    ? 'border-primary/40 bg-primary/10 text-primary'
                    : 'border-border bg-surface-elevated/50 text-muted-foreground hover:bg-surface-hover hover:text-foreground',
                )}
              >
                {tab.label}
              </button>
            ))}
          </div>

          {activeTab === 'overview' && (
            <>
              <Card className="border-border bg-card">
                <CardHeader>
                  <p className="eyebrow" style={{ margin: 0 }}>Job Description</p>
                </CardHeader>
                <CardContent>
                  <pre className="jobDescription">{job.description}</pre>
                </CardContent>
              </Card>

              {skillBadges.length > 0 && (
                <Card className="border-border bg-card">
                  <CardHeader>
                    <p className="eyebrow" style={{ margin: 0 }}>Skills & Signals</p>
                  </CardHeader>
                  <CardContent>
                    <div className="flex flex-wrap gap-2">
                      {skillBadges.map((item) => (
                        <Badge key={item} variant="muted" className="px-3 py-1 text-xs">
                          {item}
                        </Badge>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              )}
            </>
          )}

          {activeTab === 'match' && (
            <Card className="border-border bg-card">
              <CardHeader>
                <p className="eyebrow" style={{ margin: 0 }}>Match Breakdown</p>
              </CardHeader>
              <CardContent className="space-y-5">
                {fit ? (
                  <>
                    <div className="flex items-start gap-4 rounded-xl border border-primary/20 bg-primary/5 p-4">
                      <FitScoreBox score={fit.score} size="md" showLabel />
                      <p className="m-0 text-sm leading-relaxed text-muted-foreground">
                        {fit.evidence.length > 0
                          ? fit.evidence[0]
                          : 'Fit analysis is based on current profile, saved jobs, and explicit job signals.'}
                      </p>
                    </div>

                    <div>
                      <p className="mb-2 flex items-center gap-2 text-sm font-medium text-content-success">
                        <CheckCircle2 className="h-4 w-4" />
                        Strengths
                      </p>
                      {fit.evidence.length > 0 ? (
                        <ul className="space-y-2">
                          {fit.evidence.map((entry) => (
                            <li key={entry} className="flex items-start gap-2 text-sm text-muted-foreground">
                              <span className="mt-1 text-fit-excellent">•</span>
                              {entry}
                            </li>
                          ))}
                        </ul>
                      ) : (
                        <p className="text-sm text-muted-foreground">No positive signals yet.</p>
                      )}
                    </div>

                    <div className="grid gap-4 md:grid-cols-2">
                      <div>
                        <p className="mb-2 flex items-center gap-2 text-sm font-medium text-content-success">
                          <CheckCircle2 className="h-4 w-4" />
                          Matched Terms
                        </p>
                        {fit.matchedTerms.length > 0 ? (
                          <div className="flex flex-wrap gap-2">
                            {fit.matchedTerms.map((term) => (
                              <Badge key={term} variant="success" className="px-3 py-1 text-xs">
                                {term}
                              </Badge>
                            ))}
                          </div>
                        ) : (
                          <p className="text-sm text-muted-foreground">No matched terms returned.</p>
                        )}
                      </div>

                      <div>
                        <p className="mb-2 flex items-center gap-2 text-sm font-medium text-content-warning">
                          <AlertCircle className="h-4 w-4" />
                          Missing Signals
                        </p>
                        {fit.missingTerms.length > 0 ? (
                          <div className="flex flex-wrap gap-2">
                            {fit.missingTerms.map((term) => (
                              <Badge key={term} variant="danger" className="px-3 py-1 text-xs">
                                {term}
                              </Badge>
                            ))}
                          </div>
                        ) : (
                          <p className="text-sm text-muted-foreground">No missing signals returned.</p>
                        )}
                      </div>
                    </div>
                  </>
                ) : (
                  <p className="text-sm text-muted-foreground">Fit analysis is not ready yet.</p>
                )}
              </CardContent>
            </Card>
          )}

          {activeTab === 'lifecycle' && (
            <Card className="border-border bg-card">
              <CardHeader>
                <p className="eyebrow" style={{ margin: 0 }}>Lifecycle</p>
              </CardHeader>
              <CardContent>
                <div className="jobDetailFacts">
                  <div><span>вперше побачено</span><strong>{formatDate(job.firstSeenAt) ?? 'n/a'}</strong></div>
                  <div><span>останній раз</span><strong>{formatDate(job.lastSeenAt) ?? 'n/a'}</strong></div>
                  {job.inactivatedAt && (
                    <div><span>деактивовано</span><strong>{formatDate(job.inactivatedAt)}</strong></div>
                  )}
                  {job.reactivatedAt && (
                    <div><span>реактивовано</span><strong>{formatDate(job.reactivatedAt)}</strong></div>
                  )}
                  {job.primaryVariant && (
                    <div><span>source id</span><strong>{job.primaryVariant.sourceJobId}</strong></div>
                  )}
                </div>
              </CardContent>
            </Card>
          )}

        </div>

        {/* ── Sidebar ── */}
        <div className="space-y-4">

          {/* AI Fit Analysis */}
          {profileId && (
            <Card className="border-border bg-card">
              <CardHeader>
                <p className="eyebrow" style={{ margin: 0, display: 'flex', alignItems: 'center', gap: 6 }}>
                  <Sparkles size={13} /> Відповідність профілю
                </p>
              </CardHeader>
              <CardContent className="space-y-3">
                {fit ? (
                  <>
                    <FitScoreBox score={fit.score} size="lg" showLabel className="mx-auto mb-1" />

                    {fit.evidence.length > 0 && (
                      <div>
                        <p className="flex items-center gap-1.5 text-xs font-medium text-content-success mb-1.5">
                          <CheckCircle2 className="h-3.5 w-3.5" /> Evidence
                        </p>
                        <ul className="space-y-1 pl-1">
                          {fit.evidence.map((e) => (
                            <li key={e} className="text-xs text-muted-foreground flex items-start gap-1.5">
                              <span className="text-content-success mt-0.5">•</span> {e}
                            </li>
                          ))}
                        </ul>
                      </div>
                    )}

                    {fit.matchedTerms.length > 0 && (
                        <div>
                          <p className="flex items-center gap-1.5 text-xs font-medium text-content-success mb-1.5">
                            <CheckCircle2 className="h-3.5 w-3.5" /> Matched
                          </p>
                          <div className="flex flex-wrap gap-1.5">
                            {fit.matchedTerms.map((t) => (
                              <Badge key={t} variant="success" className="px-3 py-1 text-xs">
                                {t}
                              </Badge>
                            ))}
                          </div>
                      </div>
                    )}

                    {fit.missingTerms.length > 0 && (
                        <div>
                          <p className="flex items-center gap-1.5 text-xs font-medium text-content-warning mb-1.5">
                            <AlertCircle className="h-3.5 w-3.5" /> Missing
                          </p>
                          <div className="flex flex-wrap gap-1.5">
                            {fit.missingTerms.map((t) => (
                              <Badge key={t} variant="danger" className="px-3 py-1 text-xs">
                                {t}
                              </Badge>
                            ))}
                          </div>
                      </div>
                    )}
                  </>
                ) : (
                  <p className="text-sm text-muted-foreground">Аналізуємо…</p>
                )}
              </CardContent>
            </Card>
          )}

          {/* Feedback */}
          <Card className="border-border bg-card">
            <CardHeader>
              <p className="eyebrow" style={{ margin: 0 }}>Feedback</p>
            </CardHeader>
            <CardContent className="space-y-2">
              {isHidden ? (
                <Button
                  variant="outline"
                  size="sm"
                  className="w-full justify-start"
                  disabled={unhideMutation.isPending}
                  onClick={() => unhideMutation.mutate()}
                >
                  {unhideMutation.isPending ? 'Показуємо…' : 'Unhide'}
                </Button>
              ) : (
                <Button
                  variant="outline"
                  size="sm"
                  className="w-full justify-start"
                  disabled={hideMutation.isPending}
                  onClick={() => hideMutation.mutate()}
                >
                  {hideMutation.isPending ? 'Ховаємо…' : 'Hide job'}
                </Button>
              )}

              {isBadFit ? (
                <Button
                  variant="outline"
                  size="sm"
                  className="w-full justify-start"
                  disabled={unmarkBadFitMutation.isPending}
                  onClick={() => unmarkBadFitMutation.mutate()}
                >
                  {unmarkBadFitMutation.isPending ? 'Знімаємо…' : 'Remove bad fit'}
                </Button>
              ) : (
                <Button
                  variant="outline"
                  size="sm"
                  className="w-full justify-start text-destructive hover:text-destructive"
                  disabled={badFitMutation.isPending}
                  onClick={() => badFitMutation.mutate()}
                >
                  {badFitMutation.isPending ? 'Позначаємо…' : 'Not a good fit'}
                </Button>
              )}

              <Button
                variant="outline"
                size="sm"
                className={cn(
                  'w-full justify-start',
                  companyStatus === 'whitelist' && 'bg-primary/10 border-primary/30',
                )}
                disabled={companyFeedbackMutation.isPending}
                onClick={() => companyFeedbackMutation.mutate('whitelist')}
              >
                {companyStatus === 'whitelist' ? 'Unwhitelist company' : 'Whitelist company'}
              </Button>

              <Button
                variant="outline"
                size="sm"
                className={cn(
                  'w-full justify-start',
                  companyStatus === 'blacklist' && 'bg-destructive/10 border-destructive/30 text-destructive',
                )}
                disabled={companyFeedbackMutation.isPending}
                onClick={() => companyFeedbackMutation.mutate('blacklist')}
              >
                {companyStatus === 'blacklist' ? 'Unblacklist company' : 'Blacklist company'}
              </Button>
            </CardContent>
          </Card>

        </div>
      </div>
    </Page>
  );
}
