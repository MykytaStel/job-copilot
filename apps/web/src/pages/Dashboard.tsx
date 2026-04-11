import { Link } from 'react-router-dom';
import {
  ArrowRight,
  Bookmark,
  Briefcase,
  CalendarDays,
  Send,
  XCircle,
} from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import type { Application, ApplicationStatus, JobPosting } from '@job-copilot/shared';

import { getApplications, getDashboardStats, getJobs } from '../api';
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
} satisfies Record<ApplicationStatus, React.ReactElement>;

export default function Dashboard() {
  const { data: jobs = [], error: jobsError } = useQuery<JobPosting[]>({
    queryKey: queryKeys.jobs.all(),
    queryFn: getJobs,
  });

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
    <div>
      <div className="pageHeader">
        <div>
          <h1>Dashboard</h1>
          <p className="muted">
            Frontend is now reading canonical data from `engine-api`.
          </p>
        </div>
        <Link
          to="/profile"
          className="btn"
          style={{ display: 'inline-flex', alignItems: 'center', gap: 4 }}
        >
          Update Profile <ArrowRight size={14} />
        </Link>
      </div>

      {error && <p className="error">{error}</p>}

      {stats && (
        <div className="statsGrid">
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
            <div className="statNumber">{jobs.length}</div>
            <div className="statLabel">jobs indexed</div>
          </div>
          <div className="statCard">
            <div className="statNumber">{applications.length}</div>
            <div className="statLabel">applications tracked</div>
          </div>
        </div>
      )}

      {jobs.length === 0 ? (
        <p className="muted">No jobs returned by `engine-api` yet.</p>
      ) : (
        <ul className="jobList">
          {jobs.map((job) => {
            const application = applicationByJob.get(job.id);

            return (
              <li
                key={job.id}
                className="jobItem"
                style={{ flexDirection: 'column', alignItems: 'stretch', gap: 6 }}
              >
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                  }}
                >
                  <div>
                    <strong>{job.title}</strong>
                    <p style={{ margin: 0 }}>{job.company}</p>
                  </div>
                  <div className="jobItemRight">
                    {application && (
                      <span className={`statusPill status-${application.status}`}>
                        {application.status}
                      </span>
                    )}
                    <Link
                      to={`/jobs/${job.id}`}
                      className="linkBtn"
                      style={{
                        display: 'inline-flex',
                        alignItems: 'center',
                        gap: 4,
                      }}
                    >
                      Details <ArrowRight size={13} />
                    </Link>
                  </div>
                </div>
                <p className="muted" style={{ margin: 0 }}>
                  {job.description.slice(0, 220)}
                  {job.description.length > 220 ? '…' : ''}
                </p>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
