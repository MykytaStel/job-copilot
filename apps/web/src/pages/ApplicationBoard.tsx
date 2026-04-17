import { Download, MoveRight } from 'lucide-react';
import { Link } from 'react-router-dom';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type { Application, ApplicationStatus, JobPosting } from '@job-copilot/shared';

import { getApplications, getJobs, patchApplication } from '../api';
import { queryKeys } from '../queryKeys';
import { Button } from '../components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { StatusBadge } from '../components/ui/StatusBadge';

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
    <Page>
      <PageHeader
        title="Applications"
        description="Відстежуйте вакансії від збереження до оферу в єдиному pipeline."
        actions={
          applications.length > 0 ? (
            <Button onClick={() => exportCsv(applications, jobsById)}>
              <Download size={14} />
              Export CSV
            </Button>
          ) : undefined
        }
      />

      {error && (
        <EmptyState
          message={error instanceof Error ? error.message : 'Не вдалося завантажити pipeline'}
        />
      )}

      {applications.length === 0 ? (
        <EmptyState
          message="Заявок поки немає"
          description="Збережіть першу вакансію на дашборді або на сторінці вакансії."
        />
      ) : (
        <div className="grid gap-4 xl:grid-cols-5">
          {COLUMNS.map((status) => {
            const items = applications.filter((a) => a.status === status);

            return (
              <Card key={status} className="gap-4 overflow-hidden border-border bg-card/85 py-0">
                <CardHeader className="border-b border-border/70 px-4 py-4">
                  <div className="flex items-center justify-between gap-3">
                    <CardTitle className="text-sm font-semibold">
                      <StatusBadge status={status} />
                    </CardTitle>
                    <span className="rounded-full bg-white/8 px-2 py-1 text-[11px] font-medium text-muted-foreground">
                      {items.length}
                    </span>
                  </div>
                </CardHeader>
                <CardContent className="space-y-3 px-4 py-4">
                  {items.length === 0 ? (
                    <EmptyState message="Порожньо" className="px-3 py-5" />
                  ) : (
                    items.map((application) => {
                      const job = jobsById.get(application.jobId);
                      const next = NEXT_STATUS[status];

                      return (
                        <div
                          key={application.id}
                          className="rounded-2xl border border-border/70 bg-surface-elevated/50 p-3"
                        >
                          <Link
                            to={`/applications/${application.id}`}
                            className="block text-inherit no-underline"
                          >
                            <div className="text-sm font-semibold text-card-foreground">
                              {job?.title ?? application.jobId}
                            </div>
                            <p className="m-0 mt-1 text-xs text-muted-foreground">
                              {job?.company ?? 'Unknown'}
                            </p>
                            {application.appliedAt && (
                              <p className="m-0 mt-2 text-[11px] text-muted-foreground">
                                {new Date(application.appliedAt).toLocaleDateString('uk-UA')}
                              </p>
                            )}
                          </Link>

                          <div className="mt-3 flex flex-wrap gap-2">
                            {next && (
                              <Button
                                variant="ghost"
                                size="sm"
                                disabled={moveMutation.isPending}
                                onClick={() => moveMutation.mutate({ id: application.id, status: next })}
                              >
                                <MoveRight size={12} />
                                {next}
                              </Button>
                            )}
                            {status !== 'rejected' && (
                              <Button
                                variant="outline"
                                size="sm"
                                disabled={moveMutation.isPending}
                                onClick={() =>
                                  moveMutation.mutate({ id: application.id, status: 'rejected' })
                                }
                              >
                                Reject
                              </Button>
                            )}
                          </div>
                        </div>
                      );
                    })
                  )}
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}
    </Page>
  );
}
