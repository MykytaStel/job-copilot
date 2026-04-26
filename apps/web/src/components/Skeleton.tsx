import { cn } from '../lib/cn';

type SkeletonProps = {
  height?: number;
  width?: string;
  className?: string;
};

/** Single pulse placeholder with dynamic dimensions. */
export function Skeleton({ height = 20, width = '60%', className }: SkeletonProps) {
  return (
    <span
      aria-hidden="true"
      className={cn(
        'block animate-pulse rounded-lg bg-surface-muted/80',
        className,
      )}
      style={{ height, width }}
    />
  );
}

/** Full page skeleton: simulates a page header + a few content blocks. */
export function SkeletonPage() {
  return (
    <div className="space-y-6">
      <div className="space-y-3">
        <Skeleton height={18} width="20%" />
        <Skeleton height={36} width="42%" />
        <Skeleton height={18} width="55%" />
      </div>

      <div className="rounded-2xl border border-border bg-card/70 p-5">
        <Skeleton height={22} width="36%" />
        <div className="mt-5 space-y-3">
          <Skeleton height={16} width="90%" />
          <Skeleton height={16} width="76%" />
          <Skeleton height={16} width="64%" />
        </div>
      </div>

      <SkeletonList rows={3} />
    </div>
  );
}

/** List skeleton: simulates N card rows. */
export function SkeletonList({ rows = 3 }: { rows?: number }) {
  return (
    <div className="space-y-3">
      {Array.from({ length: rows }, (_, index) => (
        <div
          key={index}
          className="rounded-2xl border border-border bg-card/70 p-5"
        >
          <Skeleton height={18} width="34%" />
          <div className="mt-4 space-y-3">
            <Skeleton height={14} width="88%" />
            <Skeleton height={14} width="72%" />
            <Skeleton height={14} width="48%" />
          </div>
        </div>
      ))}
    </div>
  );
}
