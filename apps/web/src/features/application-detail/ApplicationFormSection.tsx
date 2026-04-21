import { CalendarClock } from 'lucide-react';
import type { ApplicationStatus } from '@job-copilot/shared';

import { Button } from '../../components/ui/Button';
import { formatEnumLabel } from '../../lib/format';
import { APPLICATION_STATUS_OPTIONS } from './applicationDetail.constants';
import { Panel } from './ApplicationDetailLayout';

export function ApplicationFormSection({
  status,
  dueDate,
  isPending,
  hasChanges,
  setStatus,
  setDueDate,
  clearDueDate,
  onSubmit,
}: {
  status: ApplicationStatus;
  dueDate: string;
  isPending: boolean;
  hasChanges: boolean;
  setStatus: (value: ApplicationStatus) => void;
  setDueDate: (value: string) => void;
  clearDueDate: () => void;
  onSubmit: () => void;
}) {
  return (
    <Panel
      title="Pipeline Status"
      description="Keep the application stage and due date aligned with the current process."
      icon={CalendarClock}
    >
      <form
        className="space-y-5"
        onSubmit={(event) => {
          event.preventDefault();
          onSubmit();
        }}
      >
        <div className="grid gap-4 md:grid-cols-2">
          <label>
            Status
            <select
              value={status}
              onChange={(event) => setStatus(event.target.value as ApplicationStatus)}
            >
              {APPLICATION_STATUS_OPTIONS.map((value) => (
                <option key={value} value={value}>
                  {formatEnumLabel(value)}
                </option>
              ))}
            </select>
          </label>
          <label>
            Due date
            <input
              type="date"
              value={dueDate}
              onChange={(event) => setDueDate(event.target.value)}
            />
          </label>
        </div>

        <div className="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-3">
          <p className="m-0 text-sm text-muted-foreground">
            Save only when something actually changed to keep the activity trail clean.
          </p>
          <div className="flex flex-wrap gap-2">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={clearDueDate}
              disabled={isPending || !dueDate}
            >
              Clear due date
            </Button>
            <Button type="submit" disabled={isPending || !hasChanges}>
              {isPending ? 'Saving...' : 'Save application'}
            </Button>
          </div>
        </div>
      </form>
    </Panel>
  );
}
