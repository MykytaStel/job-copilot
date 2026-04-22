import { Link } from 'react-router-dom';
import { ArrowLeft } from 'lucide-react';
import type { ApplicationDetail } from '@job-copilot/shared';

import { Badge } from '../../components/ui/Badge';
import { StatusBadge } from '../../components/ui/StatusBadge';
import { SurfaceHero } from '../../components/ui/Surface';
import { formatDate } from '../../lib/format';
import { SummaryMetric } from './ApplicationDetailLayout';

export function ApplicationHeader({ detail }: { detail: ApplicationDetail }) {
  return (
    <SurfaceHero>
      <div className="relative">
        <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-primary/10 via-accent/6 to-transparent" />
        <div className="relative space-y-6 p-7">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
            <div className="space-y-4">
              <Link
                to="/applications"
                className="inline-flex items-center gap-2 text-sm text-primary no-underline hover:underline"
              >
                <ArrowLeft className="h-4 w-4" />
                Back to board
              </Link>

              <div className="space-y-3">
                <div className="flex flex-wrap gap-2">
                  <Badge
                    variant="default"
                    className="border-0 bg-primary/15 px-2 py-0.5 text-xs text-primary"
                  >
                    Application record
                  </Badge>
                  <Badge
                    variant="muted"
                    className="px-2 py-0.5 text-[10px] uppercase tracking-wide"
                  >
                    Notes, contacts, offer, tasks
                  </Badge>
                </div>
                <div>
                  <h1 className="m-0 text-2xl font-bold text-card-foreground">{detail.job.title}</h1>
                  <p className="m-0 mt-2 text-base text-muted-foreground">{detail.job.company}</p>
                </div>
              </div>

              <div className="flex flex-wrap items-center gap-3">
                <StatusBadge status={detail.status} />
                {detail.appliedAt ? (
                  <span className="rounded-full border border-border bg-white-a05 px-3 py-1.5 text-xs text-muted-foreground">
                    Applied {formatDate(detail.appliedAt)}
                  </span>
                ) : null}
                {detail.dueDate ? (
                  <span className="rounded-full border border-border bg-white-a05 px-3 py-1.5 text-xs text-muted-foreground">
                    Due {formatDate(detail.dueDate)}
                  </span>
                ) : null}
              </div>
            </div>

            <div className="grid gap-3 sm:grid-cols-2 lg:min-w-[360px]">
              <SummaryMetric label="Contacts" value={detail.contacts.length} />
              <SummaryMetric label="Notes" value={detail.notes.length} />
              <SummaryMetric label="Tasks" value={detail.tasks.length} />
              <SummaryMetric label="Activities" value={detail.activities.length} />
            </div>
          </div>
        </div>
      </div>
    </SurfaceHero>
  );
}
