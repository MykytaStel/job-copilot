import { Link } from 'react-router-dom';
import {
  Bookmark,
  BookmarkCheck,
  Building2,
  Clock,
  DollarSign,
  EyeOff,
  ExternalLink,
  MapPin,
  Sparkles,
  ThumbsDown,
} from 'lucide-react';
import type { Application, JobPosting } from '@job-copilot/shared';
import { cn } from '../../lib/cn';
import { getJobMetaLabels } from '../../lib/jobPresentation';
import { Badge } from './Badge';
import { Card, CardContent } from './Card';
import { getFitBand } from './FitScoreBox';
import { StatusBadge } from './StatusBadge';

function metaIcon(label: string, job: JobPosting) {
  const presentation = job.presentation;

  if (label === presentation?.locationLabel) {
    return <MapPin className="h-3 w-3" />;
  }

  if (label === presentation?.salaryLabel) {
    return <DollarSign className="h-3 w-3" />;
  }

  return <Clock className="h-3 w-3" />;
}

const FIT_BAND_CLASSES: Record<string, string> = {
  excellent: 'text-fit-excellent bg-fit-excellent/15',
  good: 'text-fit-good bg-fit-good/15',
  fair: 'text-fit-fair bg-fit-fair/15',
  poor: 'text-fit-poor bg-fit-poor/15',
};

const FIT_BAND_LABELS: Record<string, string> = {
  excellent: 'Excellent',
  good: 'Good',
  fair: 'Fair',
  poor: 'Weak',
};

function JobFitBadge({ score }: { score: number }) {
  const band = getFitBand(score);
  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center rounded-lg px-3 py-1.5 shrink-0',
        FIT_BAND_CLASSES[band],
      )}
    >
      <span className="text-lg font-bold leading-none">{score}</span>
      <span className="text-[10px] font-medium uppercase tracking-wide mt-0.5">
        {FIT_BAND_LABELS[band]}
      </span>
    </div>
  );
}

// ── Source color helper ───────────────────────────────────────────────────────

const SOURCE_CLASSES: [string, string][] = [
  ['djinni', 'bg-blue-500/15 text-blue-400'],
  ['work.ua', 'bg-purple-500/15 text-purple-400'],
  ['linkedin', 'bg-blue-600/15 text-blue-300'],
  ['indeed', 'bg-purple-600/15 text-purple-300'],
];

function getSourceClass(source: string): string {
  const s = source.toLowerCase();
  return SOURCE_CLASSES.find(([key]) => s.includes(key))?.[1] ?? 'bg-muted text-muted-foreground';
}

// ── Skeleton ──────────────────────────────────────────────────────────────────

export function JobCardSkeleton({ compact = false }: { compact?: boolean }) {
  return (
    <Card className="border-border bg-card">
      <CardContent className={cn('p-4', compact && 'p-3')}>
        <div className="flex items-start justify-between gap-4">
          <div className="flex-1 space-y-3">
            <div className="flex items-start justify-between gap-2">
              <div className="space-y-2 flex-1">
                <div className="h-5 w-3/4 bg-muted rounded animate-pulse" />
                <div className="h-4 w-1/2 bg-muted rounded animate-pulse" />
              </div>
              <div className="h-12 w-14 bg-muted rounded-lg animate-pulse" />
            </div>
            <div className="flex gap-3">
              <div className="h-3 w-20 bg-muted rounded animate-pulse" />
              <div className="h-3 w-16 bg-muted rounded animate-pulse" />
            </div>
            {!compact && (
              <>
                <div className="flex gap-1.5">
                  <div className="h-5 w-16 bg-muted rounded animate-pulse" />
                  <div className="h-5 w-20 bg-muted rounded animate-pulse" />
                </div>
                <div className="h-12 w-full bg-muted rounded-lg animate-pulse" />
              </>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

// ── JobCard ───────────────────────────────────────────────────────────────────

interface JobCardProps {
  job: JobPosting;
  score?: number;
  application?: Application;
  isSaved?: boolean;
  isBadFit?: boolean;
  isPending?: boolean;
  compact?: boolean;
  onSave?: () => void;
  onHide?: () => void;
  onHideCompany?: () => void;
  onBadFit?: () => void;
  onUnmarkBadFit?: () => void;
}

export function JobCard({
  job,
  score,
  application,
  isSaved,
  isBadFit,
  isPending,
  compact = false,
  onSave,
  onHide,
  onHideCompany,
  onBadFit,
  onUnmarkBadFit,
}: JobCardProps) {
  const p = job.presentation;
  const title = p?.title ?? job.title;
  const company = p?.company ?? job.company;
  const source = p?.sourceLabel ?? job.primaryVariant?.source ?? '';
  const badges = p?.badges ?? [];
  const summary = p?.summary;
  const showSummary = Boolean(summary && !p?.summaryFallback && p?.summaryQuality !== 'weak');
  const metaLabels = getJobMetaLabels(job);

  const applicationStatus = application?.status;

  return (
    <Card
      className={cn(
        'group relative overflow-hidden border-border bg-card transition-colors hover:bg-surface-elevated',
        isBadFit && 'border-destructive/30',
        job.feedback?.hidden && 'opacity-50',
      )}
    >
      <CardContent className={cn('p-4', compact && 'p-3')}>
        <div className="flex items-start gap-4">
          <div className="flex-1 min-w-0">
            {/* Header row: title + score */}
            <div className="flex items-start justify-between gap-2 mb-2">
              <div className="flex-1 min-w-0">
                <Link
                  to={`/jobs/${job.id}`}
                  className="block hover:text-primary transition-colors no-underline"
                >
                  <h3
                    className={cn(
                      'font-semibold text-card-foreground hover:text-primary truncate transition-colors',
                      compact ? 'text-sm' : 'text-base',
                    )}
                  >
                    {title}
                  </h3>
                </Link>
                <div className="flex items-center gap-2 mt-1 text-sm text-muted-foreground">
                  <Building2 className="h-3.5 w-3.5 shrink-0" />
                  <span className="truncate">{company}</span>
                </div>
              </div>

              {score !== undefined && <JobFitBadge score={score} />}
            </div>

            {/* Meta row */}
            <div
              className={cn(
                'flex flex-wrap items-center gap-3 text-xs text-muted-foreground',
                compact ? 'mb-2' : 'mb-3',
              )}
            >
              {metaLabels.map((label) => (
                <span key={label} className="flex items-center gap-1">
                  {metaIcon(label, job)}
                  {label}
                </span>
              ))}
            </div>

            {/* Skills / badges */}
            {!compact && (badges.length > 0 || p?.descriptionQuality === 'weak') && (
              <div className="flex flex-wrap gap-1.5 mb-3">
                {badges.slice(0, 4).map((badge) => (
                  <Badge key={badge} variant="muted" className="text-xs px-2 py-0.5">
                    {badge}
                  </Badge>
                ))}
                {p?.descriptionQuality === 'weak' && (
                  <Badge variant="danger" className="text-xs px-2 py-0.5">
                    Description weak
                  </Badge>
                )}
                {badges.length > 4 && (
                  <Badge variant="muted" className="text-xs px-2 py-0.5">
                    +{badges.length - 4}
                  </Badge>
                )}
              </div>
            )}

            {/* Fit reasons / summary */}
            {!compact && showSummary && summary && (
              <div className="flex items-start gap-2 p-2 rounded-lg bg-surface-elevated/50 border border-edge-subtle mb-3">
                <Sparkles className="h-3.5 w-3.5 text-primary shrink-0 mt-0.5" />
                <p className="text-xs text-muted-foreground leading-relaxed">{summary}</p>
              </div>
            )}

            {/* Footer: source + status + hover actions */}
            <div className="flex items-center justify-between gap-2">
              <div className="flex items-center gap-2 flex-wrap">
                {source && (
                  <span
                    className={cn(
                      'text-[10px] font-medium px-2 py-0.5 rounded border border-current/20',
                      getSourceClass(source),
                    )}
                  >
                    {source}
                  </span>
                )}
                {applicationStatus && <StatusBadge status={applicationStatus} />}
                {!applicationStatus && isSaved && <StatusBadge status="saved" />}
                {isBadFit && <StatusBadge status="bad fit" />}
              </div>

              {/* Hover-revealed actions */}
              <div className="flex items-center gap-1 opacity-100 transition-opacity md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100">
                {onSave && (
                  <button
                    type="button"
                    title={isSaved ? 'Saved' : 'Save'}
                    disabled={isPending}
                    onClick={onSave}
                    className="h-7 w-7 flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-surface-hover transition-colors disabled:opacity-40"
                  >
                    {isSaved ? (
                      <BookmarkCheck className="h-4 w-4 text-primary" />
                    ) : (
                      <Bookmark className="h-4 w-4" />
                    )}
                  </button>
                )}
                {onHide && (
                  <button
                    type="button"
                    title="Hide"
                    disabled={isPending}
                    onClick={onHide}
                    className="h-7 w-7 flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-surface-hover transition-colors disabled:opacity-40"
                  >
                    <EyeOff className="h-4 w-4" />
                  </button>
                )}
                {onHideCompany && (
                  <button
                    type="button"
                    title={`Hide all from ${company}`}
                    disabled={isPending}
                    onClick={onHideCompany}
                    className="h-7 w-7 flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-surface-hover transition-colors disabled:opacity-40"
                  >
                    <Building2 className="h-4 w-4" />
                  </button>
                )}
                {isBadFit && onUnmarkBadFit ? (
                  <button
                    type="button"
                    title="Remove bad fit"
                    disabled={isPending}
                    onClick={onUnmarkBadFit}
                    className="h-7 w-7 flex items-center justify-center rounded-md text-fit-poor hover:bg-surface-hover transition-colors disabled:opacity-40"
                  >
                    <ThumbsDown className="h-4 w-4" />
                  </button>
                ) : onBadFit ? (
                  <button
                    type="button"
                    title="Mark bad fit"
                    disabled={isPending}
                    onClick={onBadFit}
                    className="h-7 w-7 flex items-center justify-center rounded-md text-muted-foreground hover:text-destructive hover:bg-surface-hover transition-colors disabled:opacity-40"
                  >
                    <ThumbsDown className="h-4 w-4" />
                  </button>
                ) : null}
                <Link
                  to={`/jobs/${job.id}`}
                  title="Open details"
                  className="h-7 w-7 flex items-center justify-center rounded-md text-muted-foreground hover:text-primary hover:bg-surface-hover transition-colors"
                >
                  <ExternalLink className="h-4 w-4" />
                </Link>
              </div>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
