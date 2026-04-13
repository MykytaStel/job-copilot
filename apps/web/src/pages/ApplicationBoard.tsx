import { Download } from 'lucide-react';
import { Link } from 'react-router-dom';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type { Application, ApplicationStatus, JobPosting } from '@job-copilot/shared';

import { getApplications, getJobs, patchApplication } from '../api';
import { queryKeys } from '../queryKeys';

const COLUMNS: ApplicationStatus[] = ['saved', 'applied', 'interview', 'offer', 'rejected'];

const NEXT_STATUS: Partial<Record<ApplicationStatus, ApplicationStatus>> = {
  saved: 'applied',
  applied: 'interview',
  interview: 'offer',
};

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
  const queryClient = useQueryClient();

  const { data: applications = [], error } = useQuery<Application[]>({
    queryKey: queryKeys.applications.all(),
    queryFn: getApplications,
  });

  const { data: jobs = [] } = useQuery<JobPosting[]>({
    queryKey: queryKeys.jobs.all(),
    queryFn: getJobs,
  });

  const jobsById = new Map(jobs.map((job) => [job.id, job]));

  const moveMutation = useMutation({
    mutationFn: ({ id, status }: { id: string; status: ApplicationStatus }) =>
      patchApplication(id, status),
    onSuccess: (updated) => {
      queryClient.setQueryData<Application[]>(
        queryKeys.applications.all(),
        (prev) => prev?.map((a) => (a.id === updated.id ? updated : a)) ?? [updated],
      );
      void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.stats() });
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  return (
    <div>
      <div className="pageHeader">
        <div>
          <h1>Applications</h1>
          <p className="muted">Відстежуйте вакансії від збереження до офферу.</p>
        </div>
        {applications.length > 0 && (
          <button
            onClick={() => exportCsv(applications, jobsById)}
            style={{ whiteSpace: 'nowrap', display: 'inline-flex', alignItems: 'center', gap: 4 }}
          >
            <Download size={14} /> Export CSV
          </button>
        )}
      </div>

      {error && (
        <p className="error">{error instanceof Error ? error.message : 'Error'}</p>
      )}

      {applications.length === 0 ? (
        <p className="muted">Збережіть першу вакансію на дашборді або на сторінці вакансії.</p>
      ) : (
        <div className="board">
          {COLUMNS.map((status) => {
            const items = applications.filter((a) => a.status === status);

            return (
              <div key={status} className="boardCol">
                <p className={`colHeader status-${status}`}>
                  {status} <span className="colCount">{items.length}</span>
                </p>

                {items.map((application) => {
                  const job = jobsById.get(application.jobId);
                  const next = NEXT_STATUS[status];

                  return (
                    <div key={application.id} className="boardCard">
                      <Link
                        to={`/applications/${application.id}`}
                        style={{ textDecoration: 'none', display: 'block' }}
                      >
                        <div className="boardCardTitle">{job?.title ?? application.jobId}</div>
                        <p className="muted boardCardCompany">{job?.company ?? 'Unknown'}</p>
                        {application.appliedAt && (
                          <p className="muted" style={{ marginBottom: 6, fontSize: 12 }}>
                            {new Date(application.appliedAt).toLocaleDateString('uk-UA')}
                          </p>
                        )}
                      </Link>

                      <div style={{ display: 'flex', gap: 6, marginTop: 8, flexWrap: 'wrap' }}>
                        {next && (
                          <button
                            className="ghostBtn"
                            style={{ fontSize: 12, padding: '3px 8px' }}
                            disabled={moveMutation.isPending}
                            onClick={() => moveMutation.mutate({ id: application.id, status: next })}
                          >
                            → {next}
                          </button>
                        )}
                        {status !== 'rejected' && (
                          <button
                            className="ghostBtn"
                            style={{ fontSize: 12, padding: '3px 8px', opacity: 0.6 }}
                            disabled={moveMutation.isPending}
                            onClick={() => moveMutation.mutate({ id: application.id, status: 'rejected' })}
                          >
                            ✕
                          </button>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
