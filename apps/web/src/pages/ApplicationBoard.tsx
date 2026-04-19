import {
  ArrowRight,
  BarChart3,
  BookmarkCheck,
  BriefcaseBusiness,
  CalendarClock,
  CircleX,
  Download,
  SearchCheck,
  Send,
  Users,
} from 'lucide-react';
import { Link } from 'react-router-dom';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type { Application, ApplicationStatus, JobPosting } from '@job-copilot/shared';
import { getJobs } from '../api/jobs';
import { getApplications, patchApplication } from '../api/applications';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { Page, PageGrid } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { StatusBadge } from '../components/ui/StatusBadge';
import { queryKeys } from '../queryKeys';
import { formatOptionalDate } from '../lib/format';

const COLUMNS: ApplicationStatus[] = ['saved', 'applied', 'interview', 'offer', 'rejected'];

const NEXT_STATUS: Partial<Record<ApplicationStatus, ApplicationStatus>> = {
  saved: 'applied',
  applied: 'interview',
  interview: 'offer',
};

const COLUMN_META: Record<
  ApplicationStatus,
  {
    description: string;
    icon: typeof SearchCheck;
  }
> = {
  saved: {
    description: 'Jobs kept in the pipeline but not yet submitted.',
    icon: BookmarkCheck,
  },
  applied: {
    description: 'Submitted roles waiting for recruiter or hiring-team response.',
    icon: Send,
  },
  interview: {
    description: 'Active conversations and evaluation loops in progress.',
    icon: Users,
  },
  offer: {
    description: 'Late-stage opportunities with concrete package discussion.',
    icon: BriefcaseBusiness,
  },
  rejected: {
    description: 'Closed opportunities that should still inform future choices.',
    icon: CircleX,
  },
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
  const rejectedCount = applications.filter(
    (application) => application.status === 'rejected',
  ).length;
  const activeCount = applications.length - rejectedCount;
  const latestUpdatedAt = applications
    .slice()
    .sort((left, right) => right.updatedAt.localeCompare(left.updatedAt))[0]?.updatedAt;

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
        description="Track saved roles, submitted applications, interview loops, offers, and closed outcomes in one operator board."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Applications' }]}
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
        <>
          <Card className="overflow-hidden border-border bg-card">
            <CardContent className="p-0">
              <div className="relative">
                <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-primary/10 via-accent/6 to-transparent" />
                <div className="relative flex flex-col gap-6 p-7 lg:flex-row lg:items-end lg:justify-between">
                  <div className="max-w-3xl space-y-3">
                    <div className="flex flex-wrap gap-2">
                      <Badge
                        variant="default"
                        className="border-0 bg-primary/15 px-2 py-0.5 text-xs text-primary"
                      >
                        Pipeline board
                      </Badge>
                      <Badge
                        variant="muted"
                        className="px-2 py-0.5 text-[10px] uppercase tracking-wide"
                      >
                        Saved, applied, interview, offer, rejected
                      </Badge>
                    </div>
                    <h2 className="m-0 text-2xl font-bold text-card-foreground lg:text-3xl">
                      Move opportunities through one consistent application workflow
                    </h2>
                    <p className="m-0 text-sm leading-7 text-muted-foreground lg:text-base">
                      This board is the action layer on top of saved jobs. Keep statuses current,
                      watch stalled columns, and open any application record for notes, contacts,
                      and offer tracking.
                    </p>
                  </div>
                  <div className="grid gap-3 sm:grid-cols-3 lg:min-w-[440px]">
                    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
                      <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-primary/15 bg-primary/10 text-primary">
                          <SearchCheck className="h-4 w-4" />
                        </div>
                        <div>
                          <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                            Active pipeline
                          </p>
                          <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">
                            {activeCount} roles
                          </p>
                        </div>
                      </div>
                    </div>
                    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
                      <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-primary/15 bg-primary/10 text-primary">
                          <BriefcaseBusiness className="h-4 w-4" />
                        </div>
                        <div>
                          <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                            Offers
                          </p>
                          <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">
                            {
                              applications.filter((application) => application.status === 'offer')
                                .length
                            }
                          </p>
                        </div>
                      </div>
                    </div>
                    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
                      <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-primary/15 bg-primary/10 text-primary">
                          <CalendarClock className="h-4 w-4" />
                        </div>
                        <div>
                          <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                            Last update
                          </p>
                          <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">
                            {formatOptionalDate(latestUpdatedAt) ?? 'n/a'}
                          </p>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          <PageGrid
            aside={
              <>
                <Card className="border-border bg-card">
                  <CardHeader className="gap-3">
                    <div className="flex items-start gap-3">
                      <div className="flex h-11 w-11 items-center justify-center rounded-2xl border border-primary/15 bg-primary/10 text-primary">
                        <BarChart3 className="h-5 w-5" />
                      </div>
                      <div>
                        <CardTitle className="text-base font-semibold">Pipeline Summary</CardTitle>
                        <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">
                          Quick read on board distribution and closure rate.
                        </p>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    {COLUMNS.map((status) => {
                      const count = applications.filter(
                        (application) => application.status === status,
                      ).length;

                      return (
                        <div
                          key={status}
                          className="flex items-center justify-between gap-3 rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-3"
                        >
                          <div className="flex items-center gap-3">
                            <StatusBadge status={status} />
                          </div>
                          <span className="text-sm font-semibold text-card-foreground">
                            {count}
                          </span>
                        </div>
                      );
                    })}
                  </CardContent>
                </Card>

                <Card className="border-border bg-card">
                  <CardHeader className="gap-3">
                    <div className="flex items-start gap-3">
                      <div className="flex h-11 w-11 items-center justify-center rounded-2xl border border-primary/15 bg-primary/10 text-primary">
                        <Users className="h-5 w-5" />
                      </div>
                      <div>
                        <CardTitle className="text-base font-semibold">Operator Notes</CardTitle>
                        <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">
                          Keep the board lean and move into detail views when coordination gets
                          real.
                        </p>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-3 text-sm leading-6 text-muted-foreground">
                    <div className="rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-3">
                      Open the application record when you need contacts, notes, tasks, or offer
                      state.
                    </div>
                    <div className="rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-3">
                      Keep `saved` small. If a role is serious, move it forward or reject it.
                    </div>
                    <div className="rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-3">
                      Rejected roles still matter. They help keep the learning loop honest.
                    </div>
                  </CardContent>
                </Card>
              </>
            }
          >
            <div className="grid gap-4 xl:grid-cols-5">
              {COLUMNS.map((status) => {
                const items = applications.filter((application) => application.status === status);
                const meta = COLUMN_META[status];

                return (
                  <Card
                    key={status}
                    className="gap-4 overflow-hidden border-border bg-card/85 py-0"
                  >
                    <CardHeader className="border-b border-border/70 px-4 py-4">
                      <div className="space-y-3">
                        <div className="flex items-center justify-between gap-3">
                          <div className="flex items-center gap-2">
                            <div className="flex h-9 w-9 items-center justify-center rounded-xl border border-primary/15 bg-primary/10 text-primary">
                              <meta.icon className="h-4 w-4" />
                            </div>
                            <CardTitle className="text-sm font-semibold">
                              <StatusBadge status={status} />
                            </CardTitle>
                          </div>
                          <span className="rounded-full bg-white/8 px-2 py-1 text-[11px] font-medium text-muted-foreground">
                            {items.length}
                          </span>
                        </div>
                        <p className="m-0 text-xs leading-6 text-muted-foreground">
                          {meta.description}
                        </p>
                      </div>
                    </CardHeader>
                    <CardContent className="space-y-3 px-4 py-4">
                      {items.length === 0 ? (
                        <EmptyState message="Порожньо" className="px-3 py-5" />
                      ) : (
                        items.map((application) => {
                          const job = jobsById.get(application.jobId);
                          const next = NEXT_STATUS[status];
                          const sourceLabel =
                            job?.presentation?.sourceLabel ??
                            job?.primaryVariant?.source ??
                            'source';
                          const summary = job?.presentation?.summary;

                          return (
                            <div
                              key={application.id}
                              className="rounded-2xl border border-border/70 bg-surface-elevated/50 p-3.5"
                            >
                              <Link
                                to={`/applications/${application.id}`}
                                className="block text-inherit no-underline"
                              >
                                <div className="flex flex-wrap items-center gap-2">
                                  <p className="m-0 text-sm font-semibold text-card-foreground">
                                    {job?.title ?? application.jobId}
                                  </p>
                                  <Badge
                                    variant="muted"
                                    className="px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
                                  >
                                    {sourceLabel}
                                  </Badge>
                                </div>
                                <p className="m-0 mt-1 text-xs text-muted-foreground">
                                  {job?.company ?? 'Unknown'}
                                </p>
                                {summary ? (
                                  <p className="m-0 mt-3 text-xs leading-6 text-muted-foreground">
                                    {summary}
                                  </p>
                                ) : null}
                                <div className="mt-3 flex flex-wrap gap-2">
                                  {application.appliedAt ? (
                                    <span className="rounded-full border border-border bg-white/[0.04] px-2.5 py-1 text-[11px] text-muted-foreground">
                                      Applied {formatOptionalDate(application.appliedAt) ?? 'n/a'}
                                    </span>
                                  ) : null}
                                  {application.dueDate ? (
                                    <span className="rounded-full border border-border bg-white/[0.04] px-2.5 py-1 text-[11px] text-muted-foreground">
                                      Due {formatOptionalDate(application.dueDate) ?? 'n/a'}
                                    </span>
                                  ) : null}
                                  <span className="rounded-full border border-border bg-white/[0.04] px-2.5 py-1 text-[11px] text-muted-foreground">
                                    Updated {formatOptionalDate(application.updatedAt) ?? 'n/a'}
                                  </span>
                                </div>
                              </Link>

                              <div className="mt-4 flex flex-wrap gap-2">
                                {next ? (
                                  <Button
                                    variant="ghost"
                                    size="sm"
                                    disabled={moveMutation.isPending}
                                    onClick={() =>
                                      moveMutation.mutate({ id: application.id, status: next })
                                    }
                                  >
                                    <ArrowRight size={12} />
                                    Move to {next}
                                  </Button>
                                ) : null}
                                {status !== 'rejected' ? (
                                  <Button
                                    variant="outline"
                                    size="sm"
                                    disabled={moveMutation.isPending}
                                    onClick={() =>
                                      moveMutation.mutate({
                                        id: application.id,
                                        status: 'rejected',
                                      })
                                    }
                                  >
                                    Reject
                                  </Button>
                                ) : null}
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
          </PageGrid>
        </>
      )}
    </Page>
  );
}
