import { useParams } from 'react-router-dom';
import { Bookmark, BookmarkCheck, Briefcase, ExternalLink, MapPin, Wifi } from 'lucide-react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type { Application } from '@job-copilot/shared';

import { createApplication, getApplications, getJob } from '../api';
import { queryKeys } from '../queryKeys';
import { SkeletonPage } from '../components/Skeleton';

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
    queryKey: queryKeys.jobs.detail(id!),
    queryFn: () => getJob(id!),
    enabled: !!id,
  });

  const { data: applications = [] } = useQuery<Application[]>({
    queryKey: queryKeys.applications.all(),
    queryFn: getApplications,
  });

  const existing = applications.find((a) => a.jobId === id);

  const saveMutation = useMutation({
    mutationFn: () => createApplication({ jobId: id!, status: 'saved' }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.applications.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.stats() });
      toast.success('Збережено в pipeline');
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
          ) : (
            <button
              onClick={() => saveMutation.mutate()}
              disabled={saveMutation.isPending}
              style={{ display: 'inline-flex', alignItems: 'center', gap: 6 }}
            >
              <Bookmark size={14} />
              {saveMutation.isPending ? 'Зберігаємо…' : 'Зберегти'}
            </button>
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
      </div>

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
