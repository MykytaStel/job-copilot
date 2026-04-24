import type { Application, ApplicationStatus, JobPosting } from '@job-copilot/shared';
import { ApplicationsTableRow } from './ApplicationsTableRow';

interface ApplicationsTableProps {
  applications: Application[];
  jobsById: Map<string, JobPosting>;
  selectedId: string | null;
  isPending: boolean;
  onSelect: (id: string) => void;
  onMove: (id: string, status: ApplicationStatus) => void;
}

export function ApplicationsTable({
  applications,
  jobsById,
  selectedId,
  isPending,
  onSelect,
  onMove,
}: ApplicationsTableProps) {
  return (
    <div className="overflow-hidden rounded-2xl border border-border bg-card">
      <table className="w-full table-fixed">
        <colgroup>
          <col className="w-28" />
          <col />
          <col className="hidden w-44 sm:table-column" />
          <col className="hidden w-28 lg:table-column" />
          <col className="hidden w-40 xl:table-column" />
          <col className="w-32" />
          <col className="w-8" />
        </colgroup>
        <thead>
          <tr className="border-b border-border/70 bg-surface-muted/60">
            <th className="px-4 py-3 text-left text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">
              Status
            </th>
            <th className="px-4 py-3 text-left text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">
              Role
            </th>
            <th className="hidden px-4 py-3 text-left text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground sm:table-cell">
              Company
            </th>
            <th className="hidden px-4 py-3 text-left text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground lg:table-cell">
              Updated
            </th>
            <th className="hidden px-4 py-3 text-left text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground xl:table-cell">
              Next Step
            </th>
            <th className="px-4 py-3 text-left text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">
              Change
            </th>
            <th className="px-2 py-3" />
          </tr>
        </thead>
        <tbody>
          {applications.map((application) => (
            <ApplicationsTableRow
              key={application.id}
              application={application}
              job={jobsById.get(application.jobId)}
              isSelected={application.id === selectedId}
              isPending={isPending}
              onSelect={() => onSelect(application.id)}
              onMove={onMove}
            />
          ))}
        </tbody>
      </table>
    </div>
  );
}
