import { BarChart3, Users } from 'lucide-react';
import type { Application } from '@job-copilot/shared';
import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/Card';
import { StatusBadge } from '../../components/ui/StatusBadge';
import { COLUMNS } from '../../features/application-board/applicationBoard.constants';

interface ApplicationBoardSidebarProps {
  applications: Application[];
}

export function ApplicationBoardSidebar({ applications }: ApplicationBoardSidebarProps) {
  return (
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
            const count = applications.filter((a) => a.status === status).length;
            return (
              <div
                key={status}
                className="flex items-center justify-between gap-3 rounded-2xl border border-border/70 bg-surface-muted px-4 py-3"
              >
                <StatusBadge status={status} />
                <span className="text-sm font-semibold text-card-foreground">{count}</span>
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
                Keep the board lean and move into detail views when coordination gets real.
              </p>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-3 text-sm leading-6 text-muted-foreground">
          <div className="rounded-2xl border border-border/70 bg-surface-muted px-4 py-3">
            Open the application record when you need contacts, notes, tasks, or offer state.
          </div>
          <div className="rounded-2xl border border-border/70 bg-surface-muted px-4 py-3">
            Keep `saved` small. If a role is serious, move it forward or reject it.
          </div>
          <div className="rounded-2xl border border-border/70 bg-surface-muted px-4 py-3">
            Rejected roles still matter. They help keep the learning loop honest.
          </div>
        </CardContent>
      </Card>
    </>
  );
}
