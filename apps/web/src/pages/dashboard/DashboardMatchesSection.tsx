import { Briefcase, CalendarDays, Search, SortAsc, TrendingUp } from 'lucide-react';
import { Link } from 'react-router-dom';
import type { DashboardPageState } from '../../features/dashboard/useDashboardPage';
import type { LifecycleFilter } from '../../features/dashboard/useDashboardPage';

import { Button } from '../../components/ui/Button';
import { Card, CardContent } from '../../components/ui/Card';
import { EmptyState } from '../../components/ui/EmptyState';
import { FilterChips } from '../../components/ui/FilterChips';
import { JobCard, JobCardSkeleton } from '../../components/ui/JobCard';
import { SectionHeader } from '../../components/ui/SectionHeader';

export function DashboardMatchesSection({
  mode,
  setSortByScore,
  search,
  setSearch,
  jobs,
  allJobs,
  jobsLoading,
  rankData,
  lifecycleOptions,
  selectedLifecycle,
  updateFilters,
  sourceOptions,
  selectedSource,
  sourcesError,
  applicationByJob,
  scoreById,
  saveMutation,
  hideMutation,
  badFitMutation,
  unmarkBadFitMutation,
}: Pick<
  DashboardPageState,
  | 'mode'
  | 'setSortByScore'
  | 'search'
  | 'setSearch'
  | 'jobs'
  | 'allJobs'
  | 'jobsLoading'
  | 'rankData'
  | 'lifecycleOptions'
  | 'selectedLifecycle'
  | 'updateFilters'
  | 'sourceOptions'
  | 'selectedSource'
  | 'sourcesError'
  | 'applicationByJob'
  | 'scoreById'
  | 'saveMutation'
  | 'hideMutation'
  | 'badFitMutation'
  | 'unmarkBadFitMutation'
>) {
  return (
    <div>
      <SectionHeader
        title="Your Job Matches"
        description="Jobs ranked by fit score, lifecycle, and your latest feedback."
        icon={Briefcase}
        action={{ label: 'Open Feedback', href: '/feedback' }}
      />

      <Card className="border-border bg-card">
        <CardContent className="space-y-5 px-6 py-6">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
            <div className="space-y-2">
              <p className="m-0 text-[11px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
                Ranking mode
              </p>
              <div className="flex flex-wrap gap-2">
                {[
                  { id: 'ranked', label: 'Ranked', icon: TrendingUp },
                  { id: 'recent', label: 'Recent', icon: CalendarDays },
                ].map((tab) => (
                  <Button
                    key={tab.id}
                    type="button"
                    variant="outline"
                    active={mode === tab.id}
                    size="sm"
                    onClick={() => setSortByScore(tab.id === 'ranked')}
                    className="rounded-full px-3.5"
                  >
                    <tab.icon className="h-3.5 w-3.5" />
                    {tab.label}
                  </Button>
                ))}
              </div>
            </div>

            {search && (
              <span className="shrink-0 rounded-full border border-border bg-white/[0.03] px-3 py-1.5 text-xs text-muted-foreground">
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

          <div className="flex flex-col gap-3 lg:flex-row lg:items-center">
            <div className="relative flex-1">
              <Search
                size={14}
                className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-content-muted"
              />
              <input
                type="search"
                placeholder="Фільтр за назвою, компанією…"
                value={search}
                onChange={(event) => setSearch(event.target.value)}
                className="h-11 rounded-xl border border-border bg-background/70"
                style={{ paddingLeft: 32 }}
              />
            </div>
            {rankData && rankData.length > 0 && (
              <Button
                variant={mode === 'ranked' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setSortByScore((value) => !value)}
                title="Сортувати за ML-релевантністю"
              >
                <SortAsc size={14} />
                Score
              </Button>
            )}
          </div>

          {sourcesError && (
            <p className="m-0 text-xs leading-6 text-muted-foreground">
              Каталог джерел недоступний — фільтр за джерелом тимчасово не працює.
            </p>
          )}
        </CardContent>
      </Card>

      <div className="space-y-3">
        {jobsLoading ? (
          <>
            <JobCardSkeleton />
            <JobCardSkeleton />
            <JobCardSkeleton />
          </>
        ) : jobs.length === 0 ? (
          <EmptyState
            message={search ? 'Нічого не знайдено' : 'Вакансій поки немає'}
            description={
              search ? 'Спробуйте змінити запит.' : 'Запустіть `pnpm scrape:djinni` або оновіть feed.'
            }
            icon={<Briefcase className="h-12 w-12" />}
          />
        ) : (
          jobs.map((job) => {
            const application = applicationByJob.get(job.id);
            const isSaved = !!(job.feedback?.saved || application);
            const isBadFit = !!job.feedback?.badFit;
            const isPendingAny =
              saveMutation.isPending ||
              hideMutation.isPending ||
              badFitMutation.isPending ||
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
                  !isSaved && !application
                    ? () =>
                        saveMutation.mutate({
                          jobId: job.id,
                          hasApplication: false,
                        })
                    : undefined
                }
                onHide={() => hideMutation.mutate(job.id)}
                onBadFit={!isBadFit ? () => badFitMutation.mutate(job.id) : undefined}
                onUnmarkBadFit={isBadFit ? () => unmarkBadFitMutation.mutate(job.id) : undefined}
              />
            );
          })
        )}
      </div>
    </div>
  );
}
