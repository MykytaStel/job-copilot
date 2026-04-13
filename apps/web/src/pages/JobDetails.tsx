import { useParams } from 'react-router-dom';
import { ExternalLink, RefreshCw, TimerReset } from 'lucide-react';
import { useQuery } from '@tanstack/react-query';

import { getJob } from '../api';
import { queryKeys } from '../queryKeys';
import { SkeletonPage } from '../components/Skeleton';

function formatTimestamp(value?: string) {
  if (!value) return 'n/a';

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;

  return new Intl.DateTimeFormat('uk-UA', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date);
}

export default function JobDetails() {
  const { id } = useParams<{ id: string }>();

  const { data: job, isLoading, error } = useQuery({
    queryKey: queryKeys.jobs.detail(id!),
    queryFn: () => getJob(id!),
    enabled: !!id,
  });

  if (isLoading) return <SkeletonPage />;
  if (!job) {
    return (
      <p className="error">
        {error instanceof Error ? error.message : 'Job not found'}
      </p>
    );
  }

  return (
    <div className="jobDetails">
      <div className="pageHeader">
        <div>
          <p className="eyebrow">Canonical Job View</p>
          <h1>{job.title}</h1>
          <p className="muted">{job.company}</p>
        </div>
        <div className={`jobStatePill jobDetailsState jobState-${job.lifecycleStage ?? 'active'}`}>
          {job.lifecycleStage ?? (job.isActive === false ? 'inactive' : 'active')}
        </div>
      </div>

      <section className="jobDetailMetaGrid">
        <div className="card">
          <p className="eyebrow">Lifecycle</p>
          <div className="jobDetailFacts">
            <div>
              <span>first seen</span>
              <strong>{formatTimestamp(job.firstSeenAt)}</strong>
            </div>
            <div>
              <span>last seen</span>
              <strong>{formatTimestamp(job.lastSeenAt)}</strong>
            </div>
            <div>
              <span>inactive at</span>
              <strong>{formatTimestamp(job.inactivatedAt)}</strong>
            </div>
            <div>
              <span>reactivated at</span>
              <strong>{formatTimestamp(job.reactivatedAt)}</strong>
            </div>
          </div>
        </div>

        <div className="card">
          <p className="eyebrow">Source Variant</p>
          {job.primaryVariant ? (
            <div className="jobDetailFacts">
              <div>
                <span>source</span>
                <strong>{job.primaryVariant.source}</strong>
              </div>
              <div>
                <span>source job id</span>
                <strong>{job.primaryVariant.sourceJobId}</strong>
              </div>
              <div>
                <span>fetched at</span>
                <strong>{formatTimestamp(job.primaryVariant.fetchedAt)}</strong>
              </div>
              <div>
                <span>source state</span>
                <strong>{job.primaryVariant.isActive ? 'active' : 'inactive'}</strong>
              </div>
              <a
                href={job.primaryVariant.sourceUrl}
                target="_blank"
                rel="noreferrer"
                className="jobVariantLink"
              >
                <ExternalLink size={14} />
                Open source page
              </a>
            </div>
          ) : (
            <p className="muted">This job does not have a persisted source variant yet.</p>
          )}
        </div>
      </section>

      <section className="card jobDetailHighlight">
        <div className="jobDetailHighlightRow">
          <span className="jobMetaChip">
            <RefreshCw size={13} />
            refresh-driven inactive/reactivation semantics enabled
          </span>
          <span className="jobMetaChip">
            <TimerReset size={13} />
            stable read-only payload from `engine-api`
          </span>
        </div>
      </section>

      <section className="card">
        <p className="eyebrow">Description</p>
        <pre className="jobDescription">{job.description}</pre>
      </section>
    </div>
  );
}
