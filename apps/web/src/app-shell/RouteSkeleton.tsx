export function RouteSkeleton() {
  return (
    <div className="space-y-8">
      <div className="space-y-3">
        <div className="h-4 w-40 rounded-full bg-white-a05" />
        <div className="h-10 w-96 max-w-full rounded-2xl bg-surface-soft" />
        <div className="h-4 w-[42rem] max-w-full rounded-full bg-white-a04" />
      </div>
      <div className="grid gap-4 md:grid-cols-3">
        {Array.from({ length: 3 }).map((_, index) => (
          <div
            key={index}
            className="h-32 rounded-[var(--radius-card)] border border-border/70 bg-card/70 animate-pulse"
          />
        ))}
      </div>
      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
        <div className="space-y-6">
          <div className="h-72 rounded-[var(--radius-card)] border border-border/70 bg-card/70 animate-pulse" />
          <div className="h-56 rounded-[var(--radius-card)] border border-border/70 bg-card/70 animate-pulse" />
        </div>
        <div className="h-80 rounded-[var(--radius-card)] border border-border/70 bg-card/70 animate-pulse" />
      </div>
    </div>
  );
}
