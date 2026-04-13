import type { ReactElement, ReactNode } from 'react';
import { Link } from 'react-router-dom';
import {
  Activity,
  ArchiveX,
  ArrowRight,
  Bookmark,
  Briefcase,
  CalendarDays,
  Clock3,
  RefreshCw,
  Send,
  XCircle,
} from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import type {
  Application,
  ApplicationStatus,
  JobFeedSummary,
  JobPosting,
} from '@job-copilot/shared';

import { getApplications, getDashboardStats, getJobsFeed } from '../api';
import { queryKeys } from '../queryKeys';

const STATUS_COLUMNS: ApplicationStatus[] = [
  'saved',
  'applied',
  'interview',
  'offer',
  'rejected',
];

const STATUS_ICONS = {
  saved: <Bookmark size={14} />,
  applied: <Send size={14} />,
  interview: <CalendarDays size={14} />,
  offer: <Briefcase size={14} />,
  rejected: <XCircle size={14} />,
} satisfies Record<ApplicationStatus, ReactElement>;

function formatTimestamp(value?: string) {
  if (!value) return 'n/a';

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;

  return new Intl.DateTimeFormat('uk-UA', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
}

function lifecycleLabel(job: JobPosting) {
  switch (job.lifecycleStage) {
    case 'reactivated':
      return 'reactivated';
    case 'inactive':
      return 'inactive';
    default:
      return job.isActive === false ? 'inactive' : 'active';
  }
}

function lifecycleClass(job: JobPosting) {
  switch (job.lifecycleStage) {
    case 'reactivated':
      return 'jobState-reactivated';
    case 'inactive':
      return 'jobState-inactive';
    default:
      return 'jobState-active';
  }
}

function SourceActivityCard({
  label,
  value,
  icon,
  tone,
}: {
  label: string;
  value: number;
  icon: ReactNode;
  tone: string;
}) {
  return (
    <div className={`sourceActivityCard ${tone}`}>
      <div className="sourceActivityIcon">{icon}</div>
      <div>
        <div className="sourceActivityValue">{value}</div>
        <div className="sourceActivityLabel">{label}</div>
      </div>
    </div>
  );
}

export default function Dashboard() {
  const { data: jobsFeed, error: jobsError } = useQuery<{
    jobs: JobPosting[];
    summary: JobFeedSummary;
  }>({
    queryKey: queryKeys.jobs.all(),
    queryFn: getJobsFeed,
  });

  const jobs = jobsFeed?.jobs ?? [];
  const jobSummary = jobsFeed?.summary;

  const { data: applications = [] } = useQuery<Application[]>({
    queryKey: queryKeys.applications.all(),
    queryFn: getApplications,
  });

  const { data: stats } = useQuery({
    queryKey: queryKeys.dashboard.stats(),
    queryFn: getDashboardStats,
  });

  const applicationByJob = new Map(applications.map((item) => [item.jobId, item]));
  const error =
    jobsError instanceof Error ? jobsError.message : jobsError ? 'Error' : null;

  return (
    <div className="dashboardPage">
      <section className="dashboardHero">
        <div>
          <p className="eyebrow">Demo Surface</p>
          <h1>Ingestion lifecycle is now visible in the product.</h1>
          <p className="muted dashboardHeroText">
            `engine-api` now serves canonical job lifecycle state, source refresh
            metadata, and reactivation signals that can later be reused by `ml`.
          </p>
        </div>
        <div className="dashboardHeroActions">
          <Link to="/profile" className="btn">
            Update Profile
          </Link>
          <Link to="/applications" className="ghostBtn">
            Review Pipeline <ArrowRight size={14} />
          </Link>
        </div>
      </section>

      {error && <p className="error">{error}</p>}

      {jobSummary && (
        <section className="sourceActivityGrid">
          <SourceActivityCard
            label="active now"
            value={jobSummary.activeJobs}
            icon={<Activity size={16} />}
            tone="tone-active"
          />
          <SourceActivityCard
            label="inactive after refresh"
            value={jobSummary.inactiveJobs}
            icon={<ArchiveX size={16} />}
            tone="tone-inactive"
          />
          <SourceActivityCard
            label="reactivated"
            value={jobSummary.reactivatedJobs}
            icon={<RefreshCw size={16} />}
            tone="tone-reactivated"
          />
          <SourceActivityCard
            label="jobs tracked"
            value={jobSummary.totalJobs}
            icon={<Briefcase size={16} />}
            tone="tone-neutral"
          />
        </section>
      )}

      {stats && (
        <section className="statsGrid">
          {STATUS_COLUMNS.map((status) => (
            <div key={status} className="statCard">
              <div className="statNumber">{stats.byStatus[status] ?? 0}</div>
              <div className="statLabel">
                <span
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 4,
                    justifyContent: 'center',
                  }}
                >
                  {STATUS_ICONS[status]}
                  {status}
                </span>
              </div>
            </div>
          ))}
          <div className="statCard">
            <div className="statNumber">{applications.length}</div>
            <div className="statLabel">applications tracked</div>
          </div>
        </section>
      )}

      {jobs.length === 0 ? (
        <p className="muted">No jobs returned by `engine-api` yet.</p>
      ) : (
        <section className="jobLifecycleGrid">
          {jobs.map((job) => {
            const application = applicationByJob.get(job.id);

            return (
              <article key={job.id} className="jobLifecycleCard">
                <div className="jobLifecycleHeader">
                  <div>
                    <div className={`jobStatePill ${lifecycleClass(job)}`}>
                      {lifecycleLabel(job)}
                    </div>
                    <h2>{job.title}</h2>
                    <p className="muted jobLifecycleCompany">{job.company}</p>
                  </div>
                  <div className="jobLifecycleActions">
                    {application && (
                      <span className={`statusPill status-${application.status}`}>
                        {application.status}
                      </span>
                    )}
                    <Link to={`/jobs/${job.id}`} className="linkBtn">
                      Details <ArrowRight size={13} />
                    </Link>
                  </div>
                </div>

                <p className="jobLifecycleDescription">
                  {job.description.slice(0, 200)}
                  {job.description.length > 200 ? '…' : ''}
                </p>

                <div className="jobMetaRow">
                  <span className="jobMetaChip">
                    <Clock3 size={13} />
                    seen {formatTimestamp(job.lastSeenAt)}
                  </span>
                  {job.primaryVariant && (
                    <span className="jobMetaChip">
                      <RefreshCw size={13} />
                      {job.primaryVariant.source}
                    </span>
                  )}
                  {job.primaryVariant?.sourceUrl && (
                    <a
                      href={job.primaryVariant.sourceUrl}
                      target="_blank"
                      rel="noreferrer"
                      className="jobMetaChip jobMetaLink"
                    >
                      source page
                    </a>
                  )}
                </div>

                <div className="jobTimeline">
                  <div>
                    <span>first seen</span>
                    <strong>{formatTimestamp(job.firstSeenAt)}</strong>
                  </div>
                  <div>
                    <span>last refresh</span>
                    <strong>{formatTimestamp(job.primaryVariant?.fetchedAt)}</strong>
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
              </article>
            );
          })}
        </section>
      )}
    </div>
  );
}
