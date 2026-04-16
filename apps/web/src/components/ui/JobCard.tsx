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
import { Badge } from './Badge';
import { Card, CardContent } from './Card';

// ── Fit score helpers ─────────────────────────────────────────────────────────

function fitBand(score: number): 'excellent' | 'good' | 'fair' | 'poor' {
  if (score >= 85) return 'excellent';
  if (score >= 70) return 'good';
  if (score >= 50) return 'fair';
  return 'poor';
}

function fitLabel(score: number): string {
  switch (fitBand(score)) {
    case 'excellent': return 'Excellent';
    case 'good':      return 'Good';
    case 'fair':      return 'Fair';
    default:          return 'Weak';
  }
}

function FitScoreBox({ score }: { score: number }) {
  const band = fitBand(score);
  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center rounded-lg px-3 py-1.5 shrink-0',
        band === 'excellent' && 'text-fit-excellent bg-fit-excellent/15',
        band === 'good'      && 'text-fit-good bg-fit-good/15',
        band === 'fair'      && 'text-fit-fair bg-fit-fair/15',
        band === 'poor'      && 'text-fit-poor bg-fit-poor/15',
      )}
    >
      <span className="text-lg font-bold leading-none">{score}</span>
      <span className="text-[10px] font-medium uppercase tracking-wide mt-0.5">
        {fitLabel(score)}
      </span>
    </div>
  );
}

// ── Source color helper ───────────────────────────────────────────────────────

function sourceClass(source: string): string {
  const s = source.toLowerCase();
  if (s.includes('djinni'))    return 'bg-blue-500/15 text-blue-400';
  if (s.includes('work.ua'))   return 'bg-purple-500/15 text-purple-400';
  if (s.includes('linkedin'))  return 'bg-blue-600/15 text-blue-300';
  if (s.includes('indeed'))    return 'bg-purple-600/15 text-purple-300';
  return 'bg-muted text-muted-foreground';
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
  onBadFit,
  onUnmarkBadFit,
}: JobCardProps) {
  const p = job.presentation;
  const title   = p?.title   ?? job.title;
  const company = p?.company ?? job.company;
  const source  = p?.sourceLabel ?? job.primaryVariant?.source ?? '';
  const badges  = p?.badges ?? [];

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

              {score !== undefined && <FitScoreBox score={score} />}
            </div>

            {/* Meta row */}
            <div className={cn('flex flex-wrap items-center gap-3 text-xs text-muted-foreground', compact ? 'mb-2' : 'mb-3')}>
              {p?.locationLabel && (
                <span className="flex items-center gap-1">
                  <MapPin className="h-3 w-3" />
                  {p.locationLabel}
                </span>
              )}
              {p?.salaryLabel && (
                <span className="flex items-center gap-1">
                  <DollarSign className="h-3 w-3" />
                  {p.salaryLabel}
                </span>
              )}
              {p?.freshnessLabel && (
                <span className="flex items-center gap-1">
                  <Clock className="h-3 w-3" />
                  {p.freshnessLabel}
                </span>
              )}
            </div>

            {/* Skills / badges */}
            {!compact && badges.length > 0 && (
              <div className="flex flex-wrap gap-1.5 mb-3">
                {badges.slice(0, 4).map((badge) => (
                  <Badge key={badge} variant="muted" className="text-xs px-2 py-0.5">
                    {badge}
                  </Badge>
                ))}
                {badges.length > 4 && (
                  <Badge variant="muted" className="text-xs px-2 py-0.5">
                    +{badges.length - 4}
                  </Badge>
                )}
              </div>
            )}

            {/* Fit reasons / summary */}
            {!compact && p?.summary && (
              <div className="flex items-start gap-2 p-2 rounded-lg bg-surface-elevated/50 border border-edge-subtle mb-3">
                <Sparkles className="h-3.5 w-3.5 text-primary shrink-0 mt-0.5" />
                <p className="text-xs text-muted-foreground leading-relaxed">{p.summary}</p>
              </div>
            )}

            {/* Footer: source + status + hover actions */}
            <div className="flex items-center justify-between gap-2">
              <div className="flex items-center gap-2 flex-wrap">
                {source && (
                  <span className={cn('text-[10px] font-medium px-2 py-0.5 rounded border border-current/20', sourceClass(source))}>
                    {source}
                  </span>
                )}
                {applicationStatus && (
                  <span className={`statusPill status-${applicationStatus}`}>
                    {applicationStatus}
                  </span>
                )}
                {!applicationStatus && isSaved && (
                  <span className="statusPill status-saved">saved</span>
                )}
                {isBadFit && (
                  <span className="statusPill status-rejected">bad fit</span>
                )}
              </div>

              {/* Hover-revealed actions */}
              <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                {onSave && (
                  <button
                    type="button"
                    title={isSaved ? 'Saved' : 'Save'}
                    disabled={isPending}
                    onClick={onSave}
                    className="h-7 w-7 flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-surface-hover transition-colors disabled:opacity-40"
                    style={{ background: 'transparent', border: 'none' }}
                  >
                    {isSaved
                      ? <BookmarkCheck className="h-4 w-4 text-primary" />
                      : <Bookmark className="h-4 w-4" />}
                  </button>
                )}
                {onHide && (
                  <button
                    type="button"
                    title="Hide"
                    disabled={isPending}
                    onClick={onHide}
                    className="h-7 w-7 flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-surface-hover transition-colors disabled:opacity-40"
                    style={{ background: 'transparent', border: 'none' }}
                  >
                    <EyeOff className="h-4 w-4" />
                  </button>
                )}
                {isBadFit && onUnmarkBadFit ? (
                  <button
                    type="button"
                    title="Remove bad fit"
                    disabled={isPending}
                    onClick={onUnmarkBadFit}
                    className="h-7 w-7 flex items-center justify-center rounded-md text-fit-poor hover:bg-surface-hover transition-colors disabled:opacity-40"
                    style={{ background: 'transparent', border: 'none' }}
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
                    style={{ background: 'transparent', border: 'none' }}
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
