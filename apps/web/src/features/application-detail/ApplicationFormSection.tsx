import { CalendarClock } from 'lucide-react';
import type { ApplicationOutcome, ApplicationStatus, RejectionStage } from '@job-copilot/shared';

import { Button } from '../../components/ui/Button';
import { formatEnumLabel } from '../../lib/format';
import {
  APPLICATION_OUTCOME_OPTIONS,
  APPLICATION_STATUS_OPTIONS,
  REJECTION_STAGE_OPTIONS,
} from './applicationDetail.constants';
import { Panel } from './ApplicationDetailLayout';

export function ApplicationFormSection({
  status,
  dueDate,
  outcome,
  outcomeDate,
  rejectionStage,
  isPending,
  hasChanges,
  setStatus,
  setDueDate,
  clearDueDate,
  setOutcome,
  setOutcomeDate,
  setRejectionStage,
  onSubmit,
}: {
  status: ApplicationStatus;
  dueDate: string;
  outcome: ApplicationOutcome | '';
  outcomeDate: string;
  rejectionStage: RejectionStage | '';
  isPending: boolean;
  hasChanges: boolean;
  setStatus: (value: ApplicationStatus) => void;
  setDueDate: (value: string) => void;
  clearDueDate: () => void;
  setOutcome: (value: ApplicationOutcome | '') => void;
  setOutcomeDate: (value: string) => void;
  setRejectionStage: (value: RejectionStage | '') => void;
  onSubmit: () => void;
}) {
  const showRejectionStage =
    outcome === 'rejected' || outcome === 'ghosted';

  return (
    <Panel
      title="Pipeline Status"
      description="Keep the application stage, outcome, and due date aligned with the current process."
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

        <div className="grid gap-4 md:grid-cols-2">
          <label>
            Outcome
            <select
              value={outcome}
              onChange={(event) => setOutcome(event.target.value as ApplicationOutcome | '')}
            >
              {APPLICATION_OUTCOME_OPTIONS.map((value) => (
                <option key={value} value={value}>
                  {value ? formatEnumLabel(value) : '— Not set —'}
                </option>
              ))}
            </select>
          </label>
          <label>
            Outcome date
            <input
              type="date"
              value={outcomeDate}
              onChange={(event) => setOutcomeDate(event.target.value)}
            />
          </label>
        </div>

        {showRejectionStage && (
          <label>
            Rejection stage
            <select
              value={rejectionStage}
              onChange={(event) => setRejectionStage(event.target.value as RejectionStage | '')}
            >
              {REJECTION_STAGE_OPTIONS.map((value) => (
                <option key={value} value={value}>
                  {value ? formatEnumLabel(value) : '— Not set —'}
                </option>
              ))}
            </select>
          </label>
        )}

        <div className="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-border/70 bg-surface-muted px-4 py-3">
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
