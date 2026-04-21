import type { ApplicationDetail, OfferStatus } from '@job-copilot/shared';
import { Handshake } from 'lucide-react';

import { Button } from '../../components/ui/Button';
import { StatusBadge } from '../../components/ui/StatusBadge';
import { formatDate, formatEnumLabel } from '../../lib/format';
import { OFFER_STATUS_OPTIONS } from './applicationDetail.constants';
import { InnerPanel, Panel } from './ApplicationDetailLayout';

export function OfferSection({
  detail,
  compensationLabel,
  status,
  min,
  max,
  currency,
  startsAt,
  notes,
  isPending,
  setStatus,
  setMin,
  setMax,
  setCurrency,
  setStartsAt,
  setNotes,
  onSubmit,
}: {
  detail: ApplicationDetail;
  compensationLabel: string | null;
  status: OfferStatus;
  min: string;
  max: string;
  currency: string;
  startsAt: string;
  notes: string;
  isPending: boolean;
  setStatus: (value: OfferStatus) => void;
  setMin: (value: string) => void;
  setMax: (value: string) => void;
  setCurrency: (value: string) => void;
  setStartsAt: (value: string) => void;
  setNotes: (value: string) => void;
  onSubmit: () => void;
}) {
  return (
    <Panel
      title="Offer Tracking"
      description="Record package details, status, and final decision context in one place."
      icon={Handshake}
    >
      {detail.offer ? (
        <div className="grid gap-4 md:grid-cols-3">
          <InnerPanel title="Status">
            <StatusBadge status={detail.offer.status} />
          </InnerPanel>
          <InnerPanel title="Compensation">
            <p className="m-0 text-sm text-card-foreground">{compensationLabel ?? 'Not set yet'}</p>
          </InnerPanel>
          <InnerPanel title="Starts at">
            <p className="m-0 text-sm text-card-foreground">
              {detail.offer.startsAt ? formatDate(detail.offer.startsAt) : 'Not set yet'}
            </p>
          </InnerPanel>
        </div>
      ) : (
        <div className="rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-3">
          <p className="m-0 text-sm text-muted-foreground">No offer saved yet.</p>
        </div>
      )}

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
              onChange={(event) => setStatus(event.target.value as OfferStatus)}
            >
              {OFFER_STATUS_OPTIONS.map((value) => (
                <option key={value} value={value}>
                  {formatEnumLabel(value)}
                </option>
              ))}
            </select>
          </label>
          <label>
            Currency
            <input
              value={currency}
              onChange={(event) => setCurrency(event.target.value)}
              placeholder="USD"
            />
          </label>
          <label>
            Compensation min
            <input
              type="number"
              min="0"
              value={min}
              onChange={(event) => setMin(event.target.value)}
              placeholder="5000"
            />
          </label>
          <label>
            Compensation max
            <input
              type="number"
              min="0"
              value={max}
              onChange={(event) => setMax(event.target.value)}
              placeholder="6500"
            />
          </label>
          <label>
            Starts at
            <input
              type="date"
              value={startsAt}
              onChange={(event) => setStartsAt(event.target.value)}
            />
          </label>
        </div>
        <label>
          Notes
          <textarea
            rows={4}
            value={notes}
            onChange={(event) => setNotes(event.target.value)}
            placeholder="Offer notes, package details, or decision context."
          />
        </label>
        <div className="flex justify-end">
          <Button type="submit" disabled={isPending}>
            {isPending ? 'Saving...' : detail.offer ? 'Update offer' : 'Save offer'}
          </Button>
        </div>
      </form>
    </Panel>
  );
}
