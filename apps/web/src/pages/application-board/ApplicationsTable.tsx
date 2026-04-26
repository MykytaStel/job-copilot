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
    <div className="w-full min-w-0 overflow-x-auto rounded-2xl border border-border bg-card/70">
      <table className="w-full min-w-[760px] border-collapse text-sm">
        <thead className="text-left text-xs uppercase tracking-[0.16em] text-muted-foreground">
          <tr className="border-b border-border">
            <th className="px-4 py-3 font-semibold">Status</th>
            <th className="px-4 py-3 font-semibold">Role</th>
            <th className="px-4 py-3 font-semibold">Company</th>
            <th className="px-4 py-3 font-semibold">Updated</th>
            <th className="px-4 py-3 font-semibold">Next Step</th>
            <th className="px-4 py-3 font-semibold">Change</th>
          </tr>
        </thead>

        <tbody>
          {applications.map((application) => (
            <ApplicationsTableRow
              key={application.id}
              application={application}
              job={jobsById.get(application.jobId)}
              isSelected={selectedId === application.id}
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
