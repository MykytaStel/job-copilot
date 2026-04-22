import type { Application, ApplicationStatus, JobPosting } from '@job-copilot/shared';
import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/Card';
import { EmptyState } from '../../components/ui/EmptyState';
import { StatusBadge } from '../../components/ui/StatusBadge';
import {
  COLUMN_META,
  COLUMNS,
} from '../../features/application-board/applicationBoard.constants';
import { ApplicationBoardCard } from './ApplicationBoardCard';

interface ApplicationBoardKanbanProps {
  applications: Application[];
  jobsById: Map<string, JobPosting>;
  isPending: boolean;
  onMove: (id: string, status: ApplicationStatus) => void;
}

export function ApplicationBoardKanban({
  applications,
  jobsById,
  isPending,
  onMove,
}: ApplicationBoardKanbanProps) {
  return (
    <div className="grid gap-4 xl:grid-cols-5">
      {COLUMNS.map((status) => {
        const items = applications.filter((a) => a.status === status);
        const meta = COLUMN_META[status];

        return (
          <Card key={status} className="gap-4 overflow-hidden border-border bg-card/85 py-0">
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
                  <span className="rounded-full bg-surface-dim px-2 py-1 text-[11px] font-medium text-muted-foreground">
                    {items.length}
                  </span>
                </div>
                <p className="m-0 text-xs leading-6 text-muted-foreground">{meta.description}</p>
              </div>
            </CardHeader>
            <CardContent className="space-y-3 px-4 py-4">
              {items.length === 0 ? (
                <EmptyState message="Порожньо" className="px-3 py-5" />
              ) : (
                items.map((application) => (
                  <ApplicationBoardCard
                    key={application.id}
                    application={application}
                    job={jobsById.get(application.jobId)}
                    status={status}
                    isPending={isPending}
                    onMove={onMove}
                  />
                ))
              )}
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}
