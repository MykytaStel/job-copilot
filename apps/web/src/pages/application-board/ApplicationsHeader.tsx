import { Search } from 'lucide-react';
import type { Application, ApplicationStatus } from '@job-copilot/shared';
import { COLUMNS } from '../../features/application-board/applicationBoard.constants';
import { StatusBadge } from '../../components/ui/StatusBadge';

interface ApplicationsHeaderProps {
  applications: Application[];
  search: string;
  filterStatus: ApplicationStatus | 'all';
  onSearch: (v: string) => void;
  onFilter: (v: ApplicationStatus | 'all') => void;
}

export function ApplicationsHeader({
  applications,
  search,
  filterStatus,
  onSearch,
  onFilter,
}: ApplicationsHeaderProps) {
  const total = applications.length;
  const countBy = (s: ApplicationStatus) => applications.filter((a) => a.status === s).length;

  return (
    <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
      <div className="flex flex-wrap items-center gap-2">
        <span className="rounded-full border border-border bg-surface-muted px-3 py-1 text-xs font-medium text-muted-foreground">
          {total} total
        </span>
        {COLUMNS.map((s) => {
          const count = countBy(s);
          if (count === 0) return null;
          return (
            <button
              key={s}
              type="button"
              onClick={() => onFilter(filterStatus === s ? 'all' : s)}
              className="flex shrink-0 items-center gap-1.5 rounded-full border border-border bg-surface-muted px-3 py-1 text-xs transition-colors hover:bg-surface-elevated focus:outline-none"
              aria-pressed={filterStatus === s}
            >
              <StatusBadge status={s} />
              <span className="font-semibold text-card-foreground">{count}</span>
            </button>
          );
        })}
      </div>

      <div className="relative w-full sm:w-64">
        <Search className="pointer-events-none absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
        <input
          type="text"
          value={search}
          onChange={(e) => onSearch(e.target.value)}
          placeholder="Search role or company…"
          className="w-full min-w-0 rounded-xl border border-border bg-surface-muted py-2 pl-9 pr-3 text-sm text-card-foreground placeholder:text-muted-foreground focus:border-primary/50 focus:outline-none focus:ring-1 focus:ring-primary/30"
        />
      </div>
    </div>
  );
}
