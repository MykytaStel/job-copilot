export function StatCardSkeleton() {
  return <div className="h-[140px] animate-pulse rounded-[24px] border border-border bg-card/80" />;
}

export function ListSkeleton({ rows = 5 }: { rows?: number }) {
  return (
    <div className="space-y-3">
      {Array.from({ length: rows }).map((_, index) => (
        <div
          key={index}
          className="h-16 animate-pulse rounded-2xl border border-border/70 bg-white/[0.04]"
        />
      ))}
    </div>
  );
}
