import { useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Bookmark, BookmarkCheck, Briefcase, ExternalLink, MapPin, Sparkles, Wifi } from 'lucide-react';
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

function readProfileId() {
  return window.localStorage.getItem('engine_api_profile_id');
}

function FitScoreBar({ score }: { score: number }) {
  const colorVar =
    score >= 60
      ? 'var(--color-text-success)'
      : score >= 35
        ? 'var(--color-text-warning)'
        : 'var(--color-text-danger)';
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
      <div style={{ flex: 1, height: 8, background: 'rgba(255,255,255,0.08)', borderRadius: 4, overflow: 'hidden' }}>
        <div style={{ width: `${score}%`, height: '100%', background: colorVar, borderRadius: 4, transition: 'width 0.4s ease' }} />
      </div>
      <span style={{ fontWeight: 700, fontSize: 18, color: colorVar, minWidth: 42, textAlign: 'right' }}>{score}%</span>
    </div>
  );
}

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

export default function JobDetails() {
  const { id } = useParams<{ id: string }>();
  const queryClient = useQueryClient();

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
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await markJobSaved(profileId, id!);

      if (!existing) {
        await createApplication({ jobId: id!, status: 'saved' });
      }
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
      if (!profileId) {
        throw new Error('Create a profile first');
      }

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
      if (!profileId) {
        throw new Error('Create a profile first');
      }

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
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      if (nextStatus === 'whitelist') {
        if (companyStatus === 'whitelist') {
          await removeCompanyWhitelist(profileId, job!.company);
        } else {
          await addCompanyWhitelist(profileId, job!.company);
        }
        return;
      }

      if (companyStatus === 'blacklist') {
        await removeCompanyBlacklist(profileId, job!.company);
      } else {
        await addCompanyBlacklist(profileId, job!.company);
      }
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
      <p className="error">
        {error instanceof Error ? error.message : 'Вакансія не знайдена'}
      </p>
    );
  }

  const salary = formatSalary(job.salaryMin, job.salaryMax, job.salaryCurrency);

  return (
    <div className="jobDetails">
      <div className="pageHeader">
        <div>
          <h1>{job.title}</h1>
          <p className="muted" style={{ fontSize: 16, marginTop: 4 }}>{job.company}</p>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexShrink: 0 }}>
          <div className={`jobStatePill jobDetailsState jobState-${job.lifecycleStage ?? 'active'}`}>
            {job.lifecycleStage ?? (job.isActive === false ? 'inactive' : 'active')}
          </div>

          {existing ? (
            <span className={`statusPill status-${existing.status}`} style={{ display: 'inline-flex', alignItems: 'center', gap: 5 }}>
              <BookmarkCheck size={13} /> {existing.status}
            </span>
          ) : isSaved ? (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => unsaveMutation.mutate()}
              disabled={unsaveMutation.isPending}
              title="Remove from saved"
            >
              <BookmarkCheck size={14} />
              {unsaveMutation.isPending ? 'Знімаємо…' : 'Unsave'}
            </Button>
          ) : (
            <Button
              onClick={() => saveMutation.mutate()}
              disabled={saveMutation.isPending}
            >
              <Bookmark size={14} />
              {saveMutation.isPending ? 'Зберігаємо…' : 'Зберегти'}
            </Button>
          )}

          {job.primaryVariant?.sourceUrl && (
            <a
              href={job.primaryVariant.sourceUrl}
              target="_blank"
              rel="noreferrer"
              className="ghostBtn"
              style={{ display: 'inline-flex', alignItems: 'center', gap: 6, textDecoration: 'none' }}
            >
              <ExternalLink size={14} /> Джерело
            </a>
          )}
        </div>
      </div>

      {/* Meta chips */}
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8, marginBottom: 20 }}>
        {salary && (
          <span className="jobMetaChip">
            <Briefcase size={13} /> {salary}
          </span>
        )}
        {job.seniority && (
          <span className="jobMetaChip" style={{ textTransform: 'capitalize' }}>
            {job.seniority}
          </span>
        )}
        {job.remoteType && (
          <span className="jobMetaChip">
            <Wifi size={13} /> {job.remoteType}
          </span>
        )}
        {job.primaryVariant?.source && (
          <span className="jobMetaChip">
            <MapPin size={13} /> {job.primaryVariant.source.replace('_', '.')}
          </span>
        )}
        {job.postedAt && (
          <span className="jobMetaChip">опубліковано {formatDate(job.postedAt)}</span>
        )}
        {isBadFit && <span className="statusPill status-rejected">bad fit</span>}
        {companyStatus === 'blacklist' && (
          <span className="statusPill status-rejected">company blacklisted</span>
        )}
        {companyStatus === 'whitelist' && (
          <span className="statusPill status-saved">company whitelisted</span>
        )}
      </div>

      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8, marginBottom: 16 }}>
        {isHidden ? (
          <Button variant="ghost" size="sm" disabled={unhideMutation.isPending} onClick={() => unhideMutation.mutate()}>
            {unhideMutation.isPending ? 'Показуємо…' : 'Unhide'}
          </Button>
        ) : (
          <Button variant="ghost" size="sm" disabled={hideMutation.isPending} onClick={() => hideMutation.mutate()}>
            {hideMutation.isPending ? 'Ховаємо…' : 'Hide'}
          </Button>
        )}
        {isBadFit ? (
          <Button variant="ghost" size="sm" disabled={unmarkBadFitMutation.isPending} onClick={() => unmarkBadFitMutation.mutate()}>
            {unmarkBadFitMutation.isPending ? 'Знімаємо…' : 'Remove bad fit'}
          </Button>
        ) : (
          <Button variant="ghost" size="sm" disabled={badFitMutation.isPending} onClick={() => badFitMutation.mutate()}>
            {badFitMutation.isPending ? 'Позначаємо…' : 'Mark bad fit'}
          </Button>
        )}
        <Button variant="ghost" size="sm" disabled={companyFeedbackMutation.isPending} onClick={() => companyFeedbackMutation.mutate('whitelist')}>
          {companyStatus === 'whitelist' ? 'Unwhitelist company' : 'Whitelist company'}
        </Button>
        <Button variant="ghost" size="sm" disabled={companyFeedbackMutation.isPending} onClick={() => companyFeedbackMutation.mutate('blacklist')}>
          {companyStatus === 'blacklist' ? 'Unblacklist company' : 'Blacklist company'}
        </Button>
      </div>

      {/* ML fit analysis */}
      {profileId && (
        <section className="card" style={{ marginBottom: 16 }}>
          <p className="eyebrow" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <Sparkles size={13} /> Відповідність профілю
          </p>
          {fit ? (
            <>
              <FitScoreBar score={fit.score} />
              {fit.evidence.length > 0 && (
                <ul style={{ margin: '10px 0 0', paddingLeft: 18, fontSize: 13, color: 'rgba(255,255,255,0.65)' }}>
                  {fit.evidence.map((e) => <li key={e}>{e}</li>)}
                </ul>
              )}
              {fit.matchedTerms.length > 0 && (
                <div style={{ marginTop: 10, display: 'flex', flexWrap: 'wrap', gap: 5 }}>
                  {fit.matchedTerms.map((t) => (
                    <span key={t} className="pill pill-success">{t}</span>
                  ))}
                </div>
              )}
              {fit.missingTerms.length > 0 && (
                <div style={{ marginTop: 8, display: 'flex', flexWrap: 'wrap', gap: 5 }}>
                  {fit.missingTerms.map((t) => (
                    <span key={t} className="pill pill-danger">{t}</span>
                  ))}
                </div>
              )}
            </>
          ) : (
            <p className="muted" style={{ margin: 0, fontSize: 13 }}>Аналізуємо…</p>
          )}
        </section>
      )}

      {/* Description */}
      <section className="card" style={{ marginBottom: 16 }}>
        <p className="eyebrow">Опис вакансії</p>
        <pre className="jobDescription">{job.description}</pre>
      </section>

      {/* Lifecycle details (collapsed, for context) */}
      <section className="card">
        <p className="eyebrow">Lifecycle</p>
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
      </section>
    </div>
  );
}
