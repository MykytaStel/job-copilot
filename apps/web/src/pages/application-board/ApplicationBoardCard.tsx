import { ArrowRight } from 'lucide-react';
import { Link } from 'react-router-dom';
import type { Application, ApplicationStatus, JobPosting } from '@job-copilot/shared';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { NEXT_STATUS } from '../../features/application-board/applicationBoard.constants';
import { formatOptionalDate } from '../../lib/format';

interface ApplicationBoardCardProps {
  application: Application;
  job?: JobPosting;
  status: ApplicationStatus;
  isPending: boolean;
  onMove: (id: string, status: ApplicationStatus) => void;
}

export function ApplicationBoardCard({
  application,
  job,
  status,
  isPending,
  onMove,
}: ApplicationBoardCardProps) {
  const next = NEXT_STATUS[status];
  const sourceLabel =
    job?.presentation?.sourceLabel ?? job?.primaryVariant?.source ?? 'source';
  const summary = job?.presentation?.summary;

  return (
    <div className="rounded-2xl border border-border/70 bg-surface-elevated/50 p-3.5">
      <Link to={`/applications/${application.id}`} className="block text-inherit no-underline">
        <div className="flex flex-wrap items-center gap-2">
          <p className="m-0 text-sm font-semibold text-card-foreground">
            {job?.title ?? application.jobId}
          </p>
          <Badge variant="muted" className="px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]">
            {sourceLabel}
          </Badge>
        </div>
        <p className="m-0 mt-1 text-xs text-muted-foreground">{job?.company ?? 'Unknown'}</p>
        {summary ? (
          <p className="m-0 mt-3 text-xs leading-6 text-muted-foreground">{summary}</p>
        ) : null}
        <div className="mt-3 flex flex-wrap gap-2">
          {application.appliedAt ? (
            <span className="rounded-full border border-border bg-white-a04 px-2.5 py-1 text-[11px] text-muted-foreground">
              Applied {formatOptionalDate(application.appliedAt) ?? 'n/a'}
            </span>
          ) : null}
          {application.dueDate ? (
            <span className="rounded-full border border-border bg-white-a04 px-2.5 py-1 text-[11px] text-muted-foreground">
              Due {formatOptionalDate(application.dueDate) ?? 'n/a'}
            </span>
          ) : null}
          <span className="rounded-full border border-border bg-white-a04 px-2.5 py-1 text-[11px] text-muted-foreground">
            Updated {formatOptionalDate(application.updatedAt) ?? 'n/a'}
          </span>
        </div>
      </Link>

      <div className="mt-4 flex flex-wrap gap-2">
        {next ? (
          <Button
            variant="ghost"
            size="sm"
            disabled={isPending}
            onClick={() => onMove(application.id, next)}
          >
            <ArrowRight size={12} />
            Move to {next}
          </Button>
        ) : null}
        {status !== 'rejected' ? (
          <Button
            variant="outline"
            size="sm"
            disabled={isPending}
            onClick={() => onMove(application.id, 'rejected')}
          >
            Reject
          </Button>
        ) : null}
      </div>
    </div>
  );
}
