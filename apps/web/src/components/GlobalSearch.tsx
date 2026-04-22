import { useEffect, useRef, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Search } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { globalSearch } from '../api/jobs';
import { queryKeys } from '../queryKeys';

function formatApplicationStatus(status: string) {
  return status.charAt(0).toUpperCase() + status.slice(1);
}

export function GlobalSearch() {
  const navigate = useNavigate();
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        setIsOpen(true);
        return;
      }

      if (event.key === 'Escape') {
        setIsOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const close = () => {
    setIsOpen(false);
  };

  const open = () => {
    setIsOpen(true);
  };

  const navigateTo = (path: string) => {
    close();
    void navigate(path);
  };

  return (
    <>
      <div className="relative max-w-md flex-1">
        <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <button
          type="button"
          onClick={open}
          className="flex h-10 w-full items-center justify-between rounded-xl border border-border bg-surface-muted pl-9 pr-3 text-left text-sm text-muted-foreground transition-colors hover:bg-surface-soft"
        >
          <span className="truncate">Search jobs and applications</span>
          <span className="rounded-md border border-border/80 px-2 py-0.5 text-[11px] font-semibold text-foreground/70">
            Cmd+K
          </span>
        </button>
      </div>

      {isOpen && <GlobalSearchDialog onClose={close} onNavigate={navigateTo} />}
    </>
  );
}

function GlobalSearchDialog({
  onClose,
  onNavigate,
}: {
  onClose: () => void;
  onNavigate: (path: string) => void;
}) {
  const inputRef = useRef<HTMLInputElement | null>(null);
  const [query, setQuery] = useState('');
  const [debouncedQuery, setDebouncedQuery] = useState('');

  useEffect(() => {
    const timeoutId = window.setTimeout(() => {
      setDebouncedQuery(query.trim());
    }, 300);

    return () => window.clearTimeout(timeoutId);
  }, [query]);

  useEffect(() => {
    const frame = window.requestAnimationFrame(() => {
      inputRef.current?.focus();
    });

    return () => window.cancelAnimationFrame(frame);
  }, []);

  const searchQuery = useQuery({
    queryKey: queryKeys.search.results(debouncedQuery),
    queryFn: () => globalSearch(debouncedQuery),
    enabled: debouncedQuery.length > 0,
    staleTime: 30_000,
  });

  const hasResults =
    (searchQuery.data?.jobs.length ?? 0) > 0 || (searchQuery.data?.applications.length ?? 0) > 0;

  return (
    <div className="fixed inset-0 z-50 bg-black/55 p-4 pt-20 backdrop-blur-sm" onClick={onClose}>
      <div
        className="mx-auto w-full max-w-2xl rounded-[var(--radius-hero)] border border-border bg-background/95 shadow-2xl"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="border-b border-border p-4">
          <div className="relative">
            <Search className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <input
              ref={inputRef}
              type="search"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search jobs and applications"
              className="h-12 w-full rounded-2xl border border-border bg-surface-muted pl-11 pr-4 text-sm text-foreground outline-none placeholder:text-muted-foreground/70 focus:border-primary/40"
            />
          </div>
        </div>

        <div className="max-h-[70vh] overflow-y-auto p-3">
          {debouncedQuery.length === 0 && (
            <p className="m-0 rounded-2xl border border-dashed border-border/80 px-4 py-8 text-center text-sm text-muted-foreground">
              Type to search across active jobs and application records.
            </p>
          )}

          {debouncedQuery.length > 0 && searchQuery.isLoading && (
            <p className="m-0 px-3 py-6 text-sm text-muted-foreground">Searching...</p>
          )}

          {debouncedQuery.length > 0 && searchQuery.isError && (
            <p className="m-0 px-3 py-6 text-sm text-destructive">
              {searchQuery.error instanceof Error ? searchQuery.error.message : 'Search failed'}
            </p>
          )}

          {debouncedQuery.length > 0 &&
            !searchQuery.isLoading &&
            !searchQuery.isError &&
            !hasResults && (
              <p className="m-0 px-3 py-6 text-sm text-muted-foreground">
                No matches for "{debouncedQuery}".
              </p>
            )}

          {(searchQuery.data?.jobs.length ?? 0) > 0 && (
            <section className="space-y-2">
              <p className="px-3 pt-1 text-[11px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
                Jobs
              </p>
              {searchQuery.data?.jobs.map((job) => (
                <button
                  key={job.id}
                  type="button"
                  onClick={() => onNavigate(`/jobs/${job.id}`)}
                  className="flex w-full flex-col rounded-2xl border border-transparent px-3 py-3 text-left transition-colors hover:border-white/5 hover:bg-white-a04"
                >
                  <span className="text-sm font-medium text-foreground">{job.title}</span>
                  <span className="mt-1 text-sm text-muted-foreground">{job.company}</span>
                  {job.presentation?.summary && (
                    <span className="mt-2 line-clamp-2 text-xs text-muted-foreground/80">
                      {job.presentation.summary}
                    </span>
                  )}
                </button>
              ))}
            </section>
          )}

          {(searchQuery.data?.applications.length ?? 0) > 0 && (
            <section className="space-y-2 pt-3">
              <p className="px-3 pt-1 text-[11px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
                Applications
              </p>
              {searchQuery.data?.applications.map((application) => (
                <button
                  key={application.id}
                  type="button"
                  onClick={() => onNavigate(`/applications/${application.id}`)}
                  className="flex w-full items-start justify-between gap-4 rounded-2xl border border-transparent px-3 py-3 text-left transition-colors hover:border-white/5 hover:bg-white-a04"
                >
                  <span className="min-w-0">
                    <span className="block truncate text-sm font-medium text-foreground">
                      {application.jobTitle}
                    </span>
                    <span className="mt-1 block text-sm text-muted-foreground">
                      {application.companyName}
                    </span>
                  </span>
                  <span className="shrink-0 rounded-full border border-border bg-white-a04 px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.12em] text-muted-foreground">
                    {formatApplicationStatus(application.status)}
                  </span>
                </button>
              ))}
            </section>
          )}
        </div>
      </div>
    </div>
  );
}
