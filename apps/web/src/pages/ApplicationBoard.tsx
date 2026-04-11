import { Download } from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import type { Application, JobPosting } from '@job-copilot/shared';

import { getApplications, getJobs } from '../api';
import { queryKeys } from '../queryKeys';

function exportCsv(applications: Application[], jobs: Map<string, JobPosting>) {
  const header = ['Company', 'Title', 'Status', 'Applied At', 'Updated At'];
  const rows = applications.map((application) => {
    const job = jobs.get(application.jobId);

    return [
      job?.company ?? '',
      job?.title ?? '',
      application.status,
      application.appliedAt ?? '',
      application.updatedAt,
    ].map((value) => `"${String(value).replace(/"/g, '""')}"`);
  });

  const csv = [header, ...rows].map((row) => row.join(',')).join('\n');
  const blob = new Blob([csv], { type: 'text/csv' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = 'applications.csv';
  anchor.click();
  URL.revokeObjectURL(url);
}

export default function ApplicationBoard() {
  const { data: applications = [], error } = useQuery<Application[]>({
    queryKey: queryKeys.applications.all(),
    queryFn: getApplications,
  });

  const { data: jobs = [] } = useQuery<JobPosting[]>({
    queryKey: queryKeys.jobs.all(),
    queryFn: getJobs,
  });

  const jobsById = new Map(jobs.map((job) => [job.id, job]));

  return (
    <div>
      <div className="pageHeader">
        <div>
          <h1>Applications</h1>
          <p className="muted">
            Read-only view from `engine-api` while application write flows are still
            being migrated.
          </p>
        </div>
        {applications.length > 0 && (
          <button
            onClick={() => exportCsv(applications, jobsById)}
            style={{
              whiteSpace: 'nowrap',
              display: 'inline-flex',
              alignItems: 'center',
              gap: 4,
            }}
          >
            <Download size={14} /> Export CSV
          </button>
        )}
      </div>

      {error && (
        <p className="error">{error instanceof Error ? error.message : 'Error'}</p>
      )}

      {applications.length === 0 ? (
        <p className="muted">No applications returned by `engine-api` yet.</p>
      ) : (
        <div className="board">
          {(['saved', 'applied', 'interview', 'offer', 'rejected'] as const).map(
            (status) => {
              const items = applications.filter(
                (application) => application.status === status,
              );

              return (
                <div key={status} className="boardCol">
                  <p className={`colHeader status-${status}`}>
                    {status} <span className="colCount">{items.length}</span>
                  </p>

                  {items.map((application) => {
                    const job = jobsById.get(application.jobId);

                    return (
                      <div key={application.id} className="boardCard">
                        <div className="boardCardTitle">
                          {job?.title ?? application.jobId}
                        </div>
                        <p className="muted boardCardCompany">
                          {job?.company ?? 'Unknown company'}
                        </p>
                        {application.appliedAt && (
                          <p className="muted" style={{ marginBottom: 0 }}>
                            Applied: {new Date(application.appliedAt).toLocaleString()}
                          </p>
                        )}
                      </div>
                    );
                  })}
                </div>
              );
            },
          )}
        </div>
      )}
    </div>
  );
}
