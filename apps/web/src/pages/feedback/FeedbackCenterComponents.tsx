/* eslint-disable react-refresh/only-export-components */

import { useState, type ReactNode } from 'react';
import type { JobPosting } from '@job-copilot/shared';
import {
  Ban,
  Bookmark,
  Building2,
  Clock3,
  EyeOff,
  Plus,
  ShieldCheck,
  ShieldOff,
  Star,
  ThumbsDown,
  Undo2,
  type LucideIcon,
} from 'lucide-react';

import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { Card, CardContent, CardHeader } from '../../components/ui/Card';
import { EmptyState } from '../../components/ui/EmptyState';
import { semanticIconFrameClass, semanticPanelClass } from '../../components/ui/semanticTone';
import { cn } from '../../lib/cn';
import { getJobMetaLabels } from '../../lib/jobPresentation';

export type FeedbackTab = 'saved' | 'hidden' | 'bad-fit' | 'companies' | 'timeline';
export type FeedbackListTone = Exclude<FeedbackTab, 'companies' | 'timeline'>;

export const FEEDBACK_TAB_META: Array<{
  id: FeedbackTab;
  label: string;
  description: string;
  icon: LucideIcon;
}> = [
  {
    id: 'saved',
    label: 'Saved',
    description: 'High-intent roles you want to revisit and potentially act on.',
    icon: Bookmark,
  },
  {
    id: 'hidden',
    label: 'Hidden',
    description: 'Suppressed roles that should stay out of the main ranking feed.',
    icon: EyeOff,
  },
  {
    id: 'bad-fit',
    label: 'Bad Fit',
    description: 'Explicit mismatches used as negative ranking evidence.',
    icon: ThumbsDown,
  },
  {
    id: 'companies',
    label: 'Companies',
    description: 'Allow and block lists that steer future ranking toward preferred employers.',
    icon: Building2,
  },
  {
    id: 'timeline',
    label: 'Timeline',
    description: 'Chronological feedback actions from newest to oldest.',
    icon: Clock3,
  },
];

const JOB_ROW_TONE_STYLES: Record<
  FeedbackListTone,
  {
    badge: 'default' | 'warning' | 'danger';
    badgeLabel: string;
    iconClass: string;
    actionClass: string;
  }
> = {
  saved: {
    badge: 'default',
    badgeLabel: 'Positive signal',
    iconClass: 'border-primary/20 bg-primary/10 text-primary',
    actionClass: 'text-primary hover:text-primary',
  },
  hidden: {
    badge: 'warning',
    badgeLabel: 'Suppressed',
    iconClass: 'border-border bg-white-a04 text-muted-foreground',
    actionClass: 'text-muted-foreground hover:text-foreground',
  },
  'bad-fit': {
    badge: 'danger',
    badgeLabel: 'Negative signal',
    iconClass: 'border-destructive/20 bg-destructive/10 text-destructive',
    actionClass: 'text-destructive hover:text-destructive',
  },
};

export function JobRow({
  job,
  tone,
  actionLabel,
  onAction,
  isPending,
  isSelected = false,
  onSelectedChange,
}: {
  job: JobPosting;
  tone: FeedbackListTone;
  actionLabel: string;
  onAction: (jobId: string) => void;
  isPending: boolean;
  isSelected?: boolean;
  onSelectedChange?: (jobId: string) => void;
}) {
  const presentation = job.presentation;
  const sourceLabel = presentation?.sourceLabel ?? job.primaryVariant?.source ?? 'source';
  const toneStyle = JOB_ROW_TONE_STYLES[tone];
  const metaItems = getJobMetaLabels(job);

  return (
    <Card className="overflow-hidden border-border bg-card">
      <CardContent className="flex flex-col gap-4 px-5 py-5 md:flex-row md:items-start md:justify-between">
        <div className="flex min-w-0 gap-4">
          {onSelectedChange ? (
            <input
              type="checkbox"
              aria-label={`Select ${presentation?.title ?? job.title}`}
              checked={isSelected}
              onChange={() => onSelectedChange(job.id)}
              className="mt-3 h-4 w-4 shrink-0 rounded border-border accent-primary"
            />
          ) : null}
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

export function Section({
  title,
  icon,
  description,
  count,
  children,
}: {
  title: string;
  icon: ReactNode;
  description?: string;
  count: number;
  children: ReactNode;
}) {
  return (
    <Card className="border-border bg-card">
      <CardHeader className="gap-3">
        <div className="flex items-center gap-3">
          <span className="flex h-10 w-10 items-center justify-center rounded-xl border border-border bg-white-a04 text-content-muted">
            {icon}
          </span>
          <div>
            <h2 className="m-0 text-[15px] font-semibold text-content">{title}</h2>
            {description ? (
              <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">{description}</p>
            ) : null}
          </div>
          <Badge variant="muted" className="ml-auto rounded-lg px-2 py-0.5 text-xs">
            {count}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">{children}</CardContent>
    </Card>
  );
}

export function CompanyPanel({
  title,
  description,
  count,
  value,
  placeholder,
  accent,
  onChange,
  onSubmit,
  isSubmitting,
  emptyMessage,
  children,
}: {
  title: string;
  description: string;
  count: number;
  value: string;
  placeholder: string;
  accent: 'success' | 'danger';
  onChange: (value: string) => void;
  onSubmit: () => void;
  isSubmitting: boolean;
  emptyMessage: string;
  children: ReactNode;
}) {
  return (
    <Card className="border-border bg-card">
      <CardHeader className="gap-3">
        <div className="flex items-center gap-2">
          <h2 className="m-0 text-base font-semibold text-card-foreground">{title}</h2>
          <Badge
            variant={accent}
            className="ml-auto px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
          >
            {count}
          </Badge>
        </div>
        <p className="m-0 text-sm leading-6 text-muted-foreground">{description}</p>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="rounded-2xl border border-border/70 bg-surface-muted p-3">
          <div className="flex gap-2">
            <input
              type="text"
              value={value}
              onChange={(event) => onChange(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') onSubmit();
              }}
              placeholder={placeholder}
              className="h-11 flex-1 rounded-xl border border-border bg-background/70 px-3"
            />
            <Button
              type="button"
              variant="outline"
              size="icon"
              className="h-11 w-11 rounded-xl"
              onClick={onSubmit}
              disabled={isSubmitting || !value.trim()}
            >
              <Plus className="h-4 w-4" />
            </Button>
          </div>
        </div>
        {count === 0 ? (
          <EmptyState message={emptyMessage} className="px-4 py-5 text-left" />
        ) : (
          <div className="flex flex-col gap-3">{children}</div>
        )}
      </CardContent>
    </Card>
  );
}

export function CompanyRow({
  companyName,
  notes,
  accent,
  badgeLabel,
  description,
  moveTitle,
  onMove,
  onRemove,
  onBulkHide,
  onNotesBlur,
  isMovePending,
  isRemovePending,
  isBulkHidePending,
  isUpdateNotesPending,
}: {
  companyName: string;
  notes: string;
  accent: 'success' | 'danger';
  badgeLabel: string;
  description: string;
  moveTitle: string;
  onMove: () => void;
  onRemove: () => void;
  onBulkHide: () => void;
  onNotesBlur: (notes: string) => void;
  isMovePending: boolean;
  isRemovePending: boolean;
  isBulkHidePending: boolean;
  isUpdateNotesPending: boolean;
}) {
  const [draftNotesState, setDraftNotesState] = useState(() => ({
    sourceNotes: notes,
    draftNotes: notes,
  }));
  const draftNotes =
    draftNotesState.sourceNotes === notes ? draftNotesState.draftNotes : notes;
  const iconClass =
    accent === 'success'
      ? semanticIconFrameClass.success
      : semanticIconFrameClass.danger;
  const rowClass =
    accent === 'success'
      ? semanticPanelClass.success
      : semanticPanelClass.danger;

  function handleNotesBlur() {
    if (draftNotes !== notes) {
      onNotesBlur(draftNotes);
    }
  }

  return (
    <Card className={cn('border', rowClass)}>
      <CardContent className="flex flex-col gap-4 px-4 py-4">
        <div className="flex items-start justify-between gap-4">
          <div className="flex min-w-0 items-start gap-3">
            <div
              className={cn(
                'flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border',
                iconClass,
              )}
            >
              <Building2 className="h-4 w-4" />
            </div>
            <div className="min-w-0">
              <div className="flex flex-wrap items-center gap-2">
                <p className="m-0 text-sm font-semibold text-foreground">{companyName}</p>
                <Badge
                  variant={accent}
                  className="px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
                >
                  {badgeLabel}
                </Badge>
              </div>
              <p className="mt-1 mb-0 text-xs leading-6 text-muted-foreground">{description}</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={onBulkHide}
              disabled={isBulkHidePending}
              title={`Hide all from ${companyName}`}
            >
              <EyeOff className="h-4 w-4" />
              Hide all
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-8 w-8 text-muted-foreground"
              onClick={onMove}
              disabled={isMovePending}
              title={moveTitle}
            >
              {accent === 'success' ? <Ban className="h-4 w-4" /> : <Star className="h-4 w-4" />}
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={onRemove}
              disabled={isRemovePending}
            >
              Remove
            </Button>
          </div>
        </div>
        <textarea
          value={draftNotes}
          onChange={(event) =>
            setDraftNotesState({
              sourceNotes: notes,
              draftNotes: event.target.value,
            })
          }
          onBlur={handleNotesBlur}
          maxLength={500}
          rows={2}
          placeholder="Add a short note..."
          disabled={isUpdateNotesPending}
          className="min-h-16 w-full resize-y rounded-xl border border-border bg-background/70 px-3 py-2 text-sm leading-5 text-foreground placeholder:text-muted-foreground"
        />
      </CardContent>
    </Card>
  );
}

export const FEEDBACK_SUMMARY_CARDS = [
  { key: 'savedJobsCount', title: 'Saved', icon: Bookmark },
  { key: 'hiddenJobsCount', title: 'Hidden', icon: EyeOff },
  { key: 'badFitJobsCount', title: 'Bad Fit', icon: ThumbsDown },
  { key: 'whitelistedCompaniesCount', title: 'Whitelisted', icon: ShieldCheck },
  { key: 'blacklistedCompaniesCount', title: 'Blacklisted', icon: ShieldOff },
] as const;
