import { Building2, Undo2 } from 'lucide-react';
import type { JobPosting } from '@job-copilot/shared';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { Card, CardContent } from '../../components/ui/Card';
import { cn } from '../../lib/cn';
import { getJobMetaLabels } from '../../lib/jobPresentation';
import { JOB_ROW_TONE_STYLES, type FeedbackListTone } from './feedbackCenter.constants';

interface FeedbackJobRowProps {
  job: JobPosting;
  tone: FeedbackListTone;
  actionLabel: string;
  onAction: (jobId: string) => void;
  isPending: boolean;
}

export function FeedbackJobRow({
  job,
  tone,
  actionLabel,
  onAction,
  isPending,
}: FeedbackJobRowProps) {
  const presentation = job.presentation;
  const sourceLabel = presentation?.sourceLabel ?? job.primaryVariant?.source ?? 'source';
  const toneStyle = JOB_ROW_TONE_STYLES[tone];
  const metaItems = getJobMetaLabels(job);

  return (
    <Card className="overflow-hidden border-border bg-card">
      <CardContent className="flex flex-col gap-4 px-5 py-5 md:flex-row md:items-start md:justify-between">
        <div className="flex min-w-0 gap-4">
          <div
            className={cn(
              'mt-0.5 flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border',
              toneStyle.iconClass,
            )}
          >
            <Building2 className="h-4 w-4" />
          </div>
          <div className="min-w-0 space-y-3">
            <div className="flex flex-wrap items-center gap-2">
              <p className="m-0 truncate text-sm font-semibold text-card-foreground md:text-base">
                {presentation?.title ?? job.title}
              </p>
              <Badge
                variant={toneStyle.badge}
                className="px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
              >
                {toneStyle.badgeLabel}
              </Badge>
              <Badge
                variant="muted"
                className="px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
              >
                {sourceLabel}
              </Badge>
            </div>
            <p className="m-0 text-sm text-muted-foreground">
              {presentation?.company ?? job.company}
            </p>
            {presentation?.summary ? (
              <p className="m-0 max-w-3xl text-sm leading-6 text-muted-foreground">
                {presentation.summary}
              </p>
            ) : null}
            {metaItems.length > 0 ? (
              <div className="flex flex-wrap gap-2">
                {metaItems.map((item) => (
                  <span
                    key={item}
                    className="inline-flex items-center rounded-full border border-border bg-white-a04 px-2.5 py-1 text-[11px] text-muted-foreground"
                  >
                    {item}
                  </span>
                ))}
              </div>
            ) : null}
          </div>
        </div>
        <div className="flex shrink-0 items-center md:pl-4">
          <Button
            variant="ghost"
            size="sm"
            className={cn('h-10 rounded-xl px-3', toneStyle.actionClass)}
            onClick={() => onAction(job.id)}
            disabled={isPending}
          >
            <Undo2 size={13} />
            {actionLabel}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
