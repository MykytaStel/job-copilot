import { ChevronRight } from 'lucide-react';
import type { Application, ApplicationStatus, JobPosting } from '@job-copilot/shared';
import { StatusBadge } from '../../components/ui/StatusBadge';
import { COLUMNS, NEXT_STATUS } from '../../features/application-board/applicationBoard.constants';
import { formatOptionalDate } from '../../lib/format';

interface ApplicationsTableRowProps {
  application: Application;
  job?: JobPosting;
  isSelected: boolean;
  isPending: boolean;
  onSelect: () => void;
  onMove: (id: string, status: ApplicationStatus) => void;
}

function nextStepLabel(application: Application): string {
  if (application.dueDate) return `Due ${formatOptionalDate(application.dueDate) ?? ''}`;
  const next = NEXT_STATUS[application.status];
  if (next) return `Move to ${next}`;
  return '—';
}

export function ApplicationsTableRow({
  application,
  job,
  isSelected,
  isPending,
  onSelect,
  onMove,
}: ApplicationsTableRowProps) {
  const title = job?.title ?? '—';
  const company = job?.company ?? '—';
  const updatedAt = formatOptionalDate(application.updatedAt) ?? '—';
  const nextStep = nextStepLabel(application);

  return (
    <tr
      onClick={onSelect}
      className={`cursor-pointer border-b border-border/60 transition-colors last:border-0 hover:bg-surface-elevated/60 ${isSelected ? 'bg-surface-elevated/80' : ''}`}
    >
      <td className="px-4 py-3">
        <StatusBadge status={application.status} />
      </td>
      <td className="px-4 py-3">
        <span className="text-sm font-medium text-card-foreground">{title}</span>
      </td>
      <td className="hidden px-4 py-3 sm:table-cell">
        <span className="text-sm text-muted-foreground">{company}</span>
      </td>
      <td className="hidden px-4 py-3 text-sm text-muted-foreground lg:table-cell">{updatedAt}</td>
      <td className="hidden px-4 py-3 xl:table-cell">
        <span className="text-xs text-muted-foreground">{nextStep}</span>
      </td>
      <td className="px-4 py-3">
        <select
          value={application.status}
          onChange={(e) => {
            e.stopPropagation();
            onMove(application.id, e.target.value as ApplicationStatus);
          }}
          onClick={(e) => e.stopPropagation()}
          disabled={isPending}
          className="rounded-lg border border-border bg-surface-muted px-2 py-1 text-xs text-card-foreground focus:outline-none focus:ring-1 focus:ring-primary/30 disabled:opacity-50"
          aria-label="Change status"
        >
          {COLUMNS.map((s) => (
            <option key={s} value={s}>
              {s.charAt(0).toUpperCase() + s.slice(1)}
            </option>
          ))}
        </select>
      </td>
      <td className="px-2 py-3 text-muted-foreground">
        <ChevronRight className="h-4 w-4" />
      </td>
    </tr>
  );
}
