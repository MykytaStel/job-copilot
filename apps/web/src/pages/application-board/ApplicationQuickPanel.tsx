import { X, ExternalLink, Loader2 } from 'lucide-react';
import { Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import type { Application, JobPosting } from '@job-copilot/shared';
import { getApplicationDetail } from '../../api/applications';
import { StatusBadge } from '../../components/ui/StatusBadge';
import { queryKeys } from '../../queryKeys';
import { formatOptionalDate } from '../../lib/format';

interface ApplicationQuickPanelProps {
  application: Application;
  job?: JobPosting;
  onClose: () => void;
}

export function ApplicationQuickPanel({ application, job, onClose }: ApplicationQuickPanelProps) {
  const { data: detail, isLoading } = useQuery({
    queryKey: queryKeys.applications.detail(application.id),
    queryFn: () => getApplicationDetail(application.id),
    staleTime: 30_000,
  });

  return (
    <div className="flex h-full w-full flex-col overflow-hidden rounded-2xl border border-border bg-card">
      <div className="flex items-start justify-between gap-3 border-b border-border/70 px-5 py-4">
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-semibold text-card-foreground">
            {job?.title ?? '—'}
          </p>
          <p className="mt-0.5 truncate text-xs text-muted-foreground">{job?.company ?? '—'}</p>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="shrink-0 rounded-lg p-1 text-muted-foreground transition-colors hover:bg-surface-elevated hover:text-card-foreground"
          aria-label="Close panel"
        >
          <X className="h-4 w-4" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto px-5 py-4 space-y-5">
        <div className="flex flex-wrap gap-2">
          <StatusBadge status={application.status} />
          {application.appliedAt && (
            <span className="rounded-full border border-border bg-surface-muted px-2.5 py-1 text-[11px] text-muted-foreground">
              Applied {formatOptionalDate(application.appliedAt)}
            </span>
          )}
          {application.dueDate && (
            <span className="rounded-full border border-border bg-amber-500/10 px-2.5 py-1 text-[11px] text-amber-400">
              Due {formatOptionalDate(application.dueDate)}
            </span>
          )}
        </div>

        {isLoading ? (
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" />
            Loading detail…
          </div>
        ) : detail ? (
          <>
            {detail.notes.length > 0 && (
              <div>
                <p className="mb-2 text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">
                  Latest note
                </p>
                <p className="rounded-xl border border-border/70 bg-surface-muted px-3 py-2.5 text-xs leading-relaxed text-card-foreground">
                  {detail.notes[detail.notes.length - 1]?.content}
                </p>
              </div>
            )}

            <div className="grid grid-cols-3 gap-3">
              <QuickStat label="Contacts" value={detail.contacts.length} />
              <QuickStat label="Notes" value={detail.notes.length} />
              <QuickStat label="Tasks" value={detail.tasks.length} />
            </div>

            {detail.offer && (
              <div>
                <p className="mb-2 text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">
                  Offer
                </p>
                <div className="rounded-xl border border-border/70 bg-surface-muted px-3 py-2.5">
                  <StatusBadge status={detail.offer.status} />
                  {(detail.offer.compensationMin || detail.offer.compensationMax) && (
                    <p className="mt-1.5 text-xs text-card-foreground">
                      {[detail.offer.compensationMin, detail.offer.compensationMax]
                        .filter(Boolean)
                        .join(' – ')}{' '}
                      {detail.offer.compensationCurrency}
                    </p>
                  )}
                </div>
              </div>
            )}

            {detail.tasks.filter((t) => !t.done).length > 0 && (
              <div>
                <p className="mb-2 text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">
                  Open tasks
                </p>
                <ul className="space-y-1">
                  {detail.tasks
                    .filter((t) => !t.done)
                    .slice(0, 3)
                    .map((task) => (
                      <li
                        key={task.id}
                        className="rounded-lg border border-border/70 bg-surface-muted px-3 py-2 text-xs text-card-foreground"
                      >
                        {task.title}
                        {task.remindAt && (
                          <span className="ml-2 text-muted-foreground">
                            {formatOptionalDate(task.remindAt)}
                          </span>
                        )}
                      </li>
                    ))}
                </ul>
              </div>
            )}
          </>
        ) : null}
      </div>

      <div className="border-t border-border/70 px-5 py-3">
        <Link
          to={`/applications/${application.id}`}
          className="flex items-center gap-1.5 text-sm font-medium text-primary hover:underline"
        >
          Open full detail
          <ExternalLink className="h-3.5 w-3.5" />
        </Link>
      </div>
    </div>
  );
}

function QuickStat({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-xl border border-border/70 bg-surface-muted px-3 py-2.5 text-center">
      <p className="text-lg font-semibold text-card-foreground">{value}</p>
      <p className="text-[11px] text-muted-foreground">{label}</p>
    </div>
  );
}
