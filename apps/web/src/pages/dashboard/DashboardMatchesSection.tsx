import { Briefcase, CalendarDays, DollarSign, Search, TrendingUp } from 'lucide-react';
import { Link } from 'react-router-dom';
import type { DashboardPageState } from '../../features/dashboard/useDashboardPage';
import type { LifecycleFilter } from '../../features/dashboard/useDashboardPage';
import { DENSITY_GAP, type SortMode } from '../../lib/displayPrefs';
import { useDisplayPrefs } from '../../lib/useDisplayPrefs';

import { Button } from '../../components/ui/Button';
import { EmptyState } from '../../components/ui/EmptyState';
import { FilterChips } from '../../components/ui/FilterChips';
import { JobCard, JobCardSkeleton } from '../../components/ui/JobCard';
import { SectionHeader } from '../../components/ui/SectionHeader';

const SORT_TABS: { id: SortMode; label: string; icon: React.ComponentType<{ className?: string }> }[] = [
  { id: 'relevance', label: 'Relevance', icon: TrendingUp },
  { id: 'date', label: 'Date', icon: CalendarDays },
  { id: 'salary', label: 'Salary', icon: DollarSign },
];

export function DashboardMatchesSection({
  profileId,
  mode,
  sortMode,
  setSortMode,
  search,
  setSearch,
  jobs,
  allJobs,
  rerankCoverage,
  jobsLoading,
  lifecycleOptions,
  selectedLifecycle,
  updateFilters,
  sourceOptions,
  selectedSource,
  notificationJobIds,
  companyFilter,
  clearContextFilters,
  sourcesError,
  applicationByJob,
  scoreById,
  saveMutation,
  hideMutation,
  undoHideMutation,
  bulkHideCompanyMutation,
  badFitMutation,
  undoBadFitMutation,
  unmarkBadFitMutation,
}: Pick<
  DashboardPageState,
  | 'profileId'
  | 'mode'
  | 'sortMode'
  | 'setSortMode'
  | 'search'
  | 'setSearch'
  | 'jobs'
  | 'allJobs'
  | 'rerankCoverage'
  | 'jobsLoading'
  | 'lifecycleOptions'
  | 'selectedLifecycle'
  | 'updateFilters'
  | 'sourceOptions'
  | 'selectedSource'
  | 'notificationJobIds'
  | 'companyFilter'
  | 'clearContextFilters'
  | 'sourcesError'
  | 'applicationByJob'
  | 'scoreById'
  | 'saveMutation'
  | 'hideMutation'
  | 'undoHideMutation'
  | 'bulkHideCompanyMutation'
  | 'badFitMutation'
  | 'undoBadFitMutation'
  | 'unmarkBadFitMutation'
>) {
  const { density } = useDisplayPrefs();
  const hasContextFilter = notificationJobIds.length > 0 || Boolean(companyFilter);

  const hasActiveFilters =
    search.trim().length > 0 ||
    !selectedLifecycle.includes('all') ||
    !selectedSource.includes('__all__') ||
    hasContextFilter;

  function clearFilters() {
    setSearch('');
    updateFilters({
      lifecycle: 'all',
      source: null,
    });
  }
  return (
    <div>
      <SectionHeader
        title="Your Job Matches"
        description="Jobs ranked by fit score, lifecycle, and your latest feedback."
        icon={Briefcase}
        action={{ label: 'Open Feedback', href: '/feedback' }}
      />

      <div className="mb-5 space-y-4 border-b border-border/70 pb-5">
          {hasContextFilter ? (
            <div className="rounded-2xl border border-primary/25 bg-primary/8 px-4 py-3">
              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div>
                  <p className="m-0 text-sm font-semibold text-card-foreground">
                    Reviewing a focused match set
                  </p>
                  <p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">
                    {notificationJobIds.length > 0
                      ? `${notificationJobIds.length} notification jobs are visible.`
                      : `Showing jobs from ${companyFilter}.`}
                  </p>
                </div>
                <Button type="button" size="sm" variant="outline" onClick={clearContextFilters}>
                  Clear context
                </Button>
              </div>
            </div>
          ) : null}

          <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
            <div className="space-y-2">
              <p className="m-0 text-[11px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
                Sort
              </p>
              <div className="flex flex-wrap gap-2">
                {SORT_TABS.map((tab) => (
                  <Button
                    key={tab.id}
                    type="button"
                    variant="outline"
                    active={sortMode === tab.id}
                    size="sm"
                    onClick={() => setSortMode(tab.id)}
                    disabled={tab.id === 'relevance' && !profileId}
                    className="rounded-full px-3.5"
                  >
                    <tab.icon className="h-3.5 w-3.5" />
                    {tab.label}
                  </Button>
                ))}
              </div>
            </div>

            {search && (
              <span className="shrink-0 rounded-full border border-border bg-surface-muted px-3 py-1.5 text-xs text-muted-foreground">
                {jobs.length}/{allJobs.length} visible
              </span>
            )}
          </div>

          <div className="space-y-2.5">
            <FilterChips
              options={lifecycleOptions}
              selected={selectedLifecycle}
              onChange={([v]) => updateFilters({ lifecycle: (v ?? 'all') as LifecycleFilter })}
            />
            <FilterChips
              options={sourceOptions}
              selected={selectedSource}
              onChange={([v]) => updateFilters({ source: v === '__all__' || !v ? null : v })}
            />
          </div>

          <div className="relative">
            <Search
              size={14}
              className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-content-muted"
            />
            <input
              type="search"
              placeholder="Фільтр за назвою, компанією…"
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              className="h-11 w-full rounded-xl border border-border bg-background/70"
              style={{ paddingLeft: 32 }}
            />
          </div>

          {sortMode === 'relevance' && rerankCoverage.isTruncated ? (
            <p className="m-0 text-xs leading-6 text-muted-foreground">
              Score sorting reranks the first {rerankCoverage.rankedJobs} feed items out of{' '}
              {rerankCoverage.totalJobs} to keep the dashboard responsive.
            </p>
          ) : null}

          {sourcesError && (
            <p className="m-0 text-xs leading-6 text-muted-foreground">
              Каталог джерел недоступний — фільтр за джерелом тимчасово не працює.
            </p>
          )}

          {!profileId && (
            <div className="rounded-2xl border border-border/70 bg-surface-muted px-4 py-4">
              <p className="m-0 text-sm font-medium text-card-foreground">
                Create a profile to unlock fit ranking and feedback actions
              </p>
              <p className="m-0 mt-2 text-sm leading-6 text-muted-foreground">
                You can still browse the recent feed here, but save, hide, bad-fit feedback, and
                profile-based reranking stay disabled until the active profile exists.
              </p>
              <Link to="/profile" className="mt-3 inline-flex no-underline">
                <Button size="sm">Open Profile &amp; Search</Button>
              </Link>
            </div>
          )}
      </div>

      <div className={DENSITY_GAP[density]}>
        {jobsLoading ? (
          <div className={density === 'compact' ? 'space-y-2' : 'space-y-3'}>
            {Array.from({ length: 5 }, (_, index) => (
              <JobCardSkeleton key={index} compact={density === 'compact'} />
            ))}
          </div>
        ) : jobs.length === 0 ? (
          <EmptyState
            message={hasActiveFilters ? 'No jobs found. Try adjusting your filters.' : 'No jobs available yet.'}
            description={
              hasActiveFilters
                ? 'Clear the search query or choose broader lifecycle/source filters.'
                : 'Run ingestion or refresh the feed to populate the dashboard.'
            }
            icon={<Briefcase className="h-12 w-12" />}
            action={
              hasActiveFilters ? (
                <Button type="button" size="sm" variant="outline" onClick={clearFilters}>
                  Clear filters
                </Button>
              ) : (
                <Link to="/profile" className="inline-flex no-underline">
                  <Button size="sm">Add filters</Button>
                </Link>
              )
            }
          />
        ) : (
          jobs.map((job) => {
            const application = applicationByJob.get(job.id);
            const isSaved = !!(job.feedback?.saved || application);
            const isBadFit = !!job.feedback?.badFit;
            const isPendingAny =
              saveMutation.isPending ||
              hideMutation.isPending ||
              undoHideMutation.isPending ||
              bulkHideCompanyMutation.isPending ||
              badFitMutation.isPending ||
              undoBadFitMutation.isPending ||
              unmarkBadFitMutation.isPending;

            return (
              <JobCard
                key={job.id}
                job={job}
                score={scoreById.get(job.id)}
                application={application}
                isSaved={isSaved}
                isBadFit={isBadFit}
                isPending={isPendingAny}
                onSave={
                  profileId && !isSaved && !application
                    ? () =>
                        saveMutation.mutate({
                          jobId: job.id,
                          hasApplication: false,
                        })
                    : undefined
                }
                onHide={profileId ? () => hideMutation.mutate(job.id) : undefined}
                onHideCompany={
                  profileId ? () => bulkHideCompanyMutation.mutate(job.company) : undefined
                }
                onBadFit={
                  profileId && !isBadFit
                    ? (reason) => badFitMutation.mutate({ jobId: job.id, reason })
                    : undefined
                }
                onUnmarkBadFit={
                  profileId && isBadFit ? () => unmarkBadFitMutation.mutate(job.id) : undefined
                }
              />
            );
          })
        )}
      </div>
    </div>
  );
}
