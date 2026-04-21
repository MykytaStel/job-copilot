export function AppShellProfileBadge({
  loading,
  name,
  email,
}: {
  loading: boolean;
  name?: string | null;
  email?: string | null;
}) {
  return (
    <div className="flex items-center gap-3">
      <div className="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-full bg-sidebar-accent text-sidebar-primary">
        <span className="text-sm font-medium">JC</span>
      </div>
      <div className="min-w-0 flex-1">
        {loading ? (
          <>
            <div className="mb-1 h-3.5 w-24 animate-pulse rounded bg-sidebar-accent" />
            <div className="h-3 w-32 animate-pulse rounded bg-sidebar-accent" />
          </>
        ) : (
          <>
            <p className="truncate text-sm font-medium text-sidebar-foreground">{name ?? '—'}</p>
            <p className="truncate text-xs text-sidebar-foreground/60">{email ?? '—'}</p>
          </>
        )}
      </div>
    </div>
  );
}
