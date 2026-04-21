import { Link } from 'react-router-dom';
import {
  ArrowRight,
  Bookmark,
  Briefcase,
  CalendarDays,
  KanbanSquare,
  Search,
  Send,
  SortAsc,
  Sparkles,
  TrendingUp,
  XCircle,
  Zap,
} from 'lucide-react';

import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card, CardContent, CardHeader } from '../components/ui/Card';
import { AIInsightPanel } from '../components/ui/AIInsightPanel';
import { EmptyState } from '../components/ui/EmptyState';
import { FilterChips } from '../components/ui/FilterChips';
import { JobCard, JobCardSkeleton } from '../components/ui/JobCard';
import { Page, PageGrid } from '../components/ui/Page';
import { SectionHeader } from '../components/ui/SectionHeader';
import { StatCard } from '../components/ui/StatCard';
import {
  STATUS_COLUMNS,
  STATUS_ICONS,
  type LifecycleFilter,
  useDashboardPage,
} from '../features/dashboard/useDashboardPage';

export default function Dashboard() {
  const {
    search,
    setSearch,
    sortByScore,
    setSortByScore,
    lifecycleFilter,
    updateFilters,
    jobsLoading,
    jobs,
    allJobs,
    jobSummary,
    sourcesError,
    applications,
    stats,
    rankData,
    scoreById,
    applicationByJob,
    error,
    lifecycleOptions,
    sourceOptions,
    selectedLifecycle,
    selectedSource,
    topSource,
    interviewedCount,
    mode,
    insights,
    saveMutation,
    hideMutation,
    badFitMutation,
    unmarkBadFitMutation,
  } = useDashboardPage();

  return (
    <Page>
      <Card className="overflow-hidden border-border bg-card">
        <CardContent className="p-0">
          <div className="relative">
            <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-primary/8 via-accent/6 to-transparent" />
            <div className="relative p-7 lg:p-8">
              <div className="flex flex-col gap-7 xl:flex-row xl:items-end xl:justify-between">
                <div className="max-w-3xl space-y-4">
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge
                      variant="default"
                      className="border-0 bg-primary/15 px-2 py-0.5 text-xs text-primary"
                    >
                      <Zap className="mr-1 h-3 w-3" />
                      Job Copilot UA
                    </Badge>
                    <Badge
                      variant="muted"
                      className="px-2 py-0.5 text-[10px] uppercase tracking-wide"
                    >
                      {mode === 'ranked' ? 'Ranked mode' : 'Recent mode'}
                    </Badge>
                  </div>

                  <div className="space-y-2">
                    <h1 className="m-0 text-3xl font-bold leading-tight text-card-foreground lg:text-4xl">
                      Відстежуйте вакансії, fit і pipeline в одному quiet dashboard
                    </h1>
                    <p className="m-0 max-w-2xl text-sm leading-7 text-muted-foreground lg:text-base">
                      Вакансії автоматично збираються з Djinni та Work.ua, а ranking і feedback
                      допомагають швидко знайти наступний крок.
                    </p>
                  </div>

                  <div className="grid gap-3 sm:grid-cols-3">
                    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
                      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                        Active jobs
                      </p>
                      <p className="m-0 mt-2 text-2xl font-bold text-card-foreground">
                        {jobSummary?.activeJobs ?? allJobs.length}
                      </p>
                    </div>
                    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
                      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                        Tracked pipeline
                      </p>
                      <p className="m-0 mt-2 text-2xl font-bold text-card-foreground">
                        {applications.length}
                      </p>
                    </div>
                    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
                      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                        Source focus
                      </p>
                      <p className="m-0 mt-2 text-lg font-semibold text-card-foreground">
                        {topSource}
                      </p>
                    </div>
                  </div>
                </div>

                <div className="flex flex-col gap-3 xl:min-w-[240px] xl:items-end">
                  <Link to="/profile">
                    <Button className="w-full justify-center gap-2 xl:min-w-[210px]">
                      <Sparkles className="h-4 w-4" />
                      Update Profile
                    </Button>
                  </Link>
                  <Link to="/applications">
                    <Button
                      variant="outline"
                      className="w-full justify-center gap-2 xl:min-w-[210px]"
                    >
                      Review Pipeline
                      <ArrowRight size={14} />
                    </Button>
                  </Link>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {error && <p className="error">{error}</p>}

      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <StatCard
          title="Активних вакансій"
          value={jobSummary?.activeJobs ?? allJobs.length}
          description="зараз у базі"
          icon={Briefcase}
        />
        <StatCard
          title="Збережено"
          value={stats?.byStatus.saved ?? 0}
          description="у pipeline"
          icon={Bookmark}
        />
        <StatCard
          title="Подано заявки"
          value={stats?.byStatus.applied ?? 0}
          description="готові до follow-up"
          icon={Send}
        />
        <StatCard
          title="Інтерв'ю"
          value={interviewedCount}
          description="interview + offer"
          icon={CalendarDays}
        />
      </div>

      <PageGrid
        aside={
          <>
            <AIInsightPanel insights={insights} />

            <Card className="border-border bg-card">
              <CardHeader className="gap-3">
                <div className="flex items-start gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-2xl border border-primary/15 bg-primary/10 text-primary">
                    <Sparkles className="h-4 w-4" />
                  </div>
                  <div>
                    <h2 className="m-0 text-base font-semibold text-card-foreground">
                      Quick Actions
                    </h2>
                    <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">
                      Jump into the profile, feedback, analytics, or application workflow.
                    </p>
                  </div>
                </div>
              </CardHeader>
              <CardContent className="space-y-2.5">
                <Link to="/profile" className="block no-underline">
                  <Button variant="ghost" className="w-full justify-start gap-2 text-sm">
                    <Sparkles className="h-4 w-4 text-primary" />
                    Update Search Profile
                  </Button>
                </Link>
                <Link to="/feedback" className="block no-underline">
                  <Button variant="ghost" className="w-full justify-start gap-2 text-sm">
                    <Bookmark className="h-4 w-4 text-primary" />
                    Review Saved Jobs
                  </Button>
                </Link>
                <Link to="/analytics" className="block no-underline">
                  <Button variant="ghost" className="w-full justify-start gap-2 text-sm">
                    <TrendingUp className="h-4 w-4 text-primary" />
                    View Analytics
                  </Button>
                </Link>
                <Link to="/applications" className="block no-underline">
                  <Button variant="ghost" className="w-full justify-start gap-2 text-sm">
                    <KanbanSquare className="h-4 w-4 text-primary" />
                    Application Board
                  </Button>
                </Link>
              </CardContent>
            </Card>

            {stats && (
              <Card className="border-border bg-card">
                <CardHeader className="gap-3">
                  <div className="flex items-start gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-2xl border border-primary/15 bg-primary/10 text-primary">
                      <XCircle className="h-4 w-4" />
                    </div>
                    <div>
                      <h2 className="m-0 text-base font-semibold text-card-foreground">Pipeline</h2>
                      <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">
                        Current distribution of tracked applications by stage.
                      </p>
                    </div>
                  </div>
                </CardHeader>
                <CardContent className="space-y-3.5">
                  {STATUS_COLUMNS.map((status) => {
                    const StatusIcon = STATUS_ICONS[status];

                    return (
                      <div key={status} className="flex items-center justify-between text-sm">
                        <span className="flex items-center gap-1.5 text-muted-foreground">
                          <StatusIcon size={14} />
                          {status}
                        </span>
                        <span className="font-medium text-card-foreground">
                          {stats.byStatus[status] ?? 0}
                        </span>
                      </div>
                    );
                  })}
                  <div className="flex items-center justify-between border-t border-border pt-3 text-sm">
                    <span className="text-muted-foreground">total tracked</span>
                    <span className="font-medium text-card-foreground">{applications.length}</span>
                  </div>
                </CardContent>
              </Card>
            )}

            {jobSummary && (
              <Card className="border-border bg-card">
                <CardHeader className="gap-3">
                  <div className="flex items-start gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-2xl border border-primary/15 bg-primary/10 text-primary">
                      <Zap className="h-4 w-4" />
                    </div>
                    <div>
                      <h2 className="m-0 text-base font-semibold text-card-foreground">Feed</h2>
                      <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">
                        Inventory health across active, reactivated, and inactive jobs.
                      </p>
                    </div>
                  </div>
                </CardHeader>
                <CardContent className="space-y-3.5">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">Active</span>
                    <span className="font-medium text-fit-excellent">{jobSummary.activeJobs}</span>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">Reactivated</span>
                    <span className="font-medium text-fit-good">{jobSummary.reactivatedJobs}</span>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">Inactive</span>
                    <span className="font-medium text-muted-foreground">
                      {jobSummary.inactiveJobs}
                    </span>
                  </div>
                  <div className="flex items-center justify-between border-t border-border pt-3 text-sm">
                    <span className="text-muted-foreground">Total tracked</span>
                    <span className="font-medium text-card-foreground">{jobSummary.totalJobs}</span>
                  </div>
                </CardContent>
              </Card>
            )}
          </>
        }
      >
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
                    variant={sortByScore ? 'default' : 'outline'}
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
                  search
                    ? 'Спробуйте змінити запит.'
                    : 'Запустіть `pnpm scrape:djinni` або оновіть feed.'
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
                    onUnmarkBadFit={
                      isBadFit ? () => unmarkBadFitMutation.mutate(job.id) : undefined
                    }
                  />
                );
              })
            )}
          </div>
        </div>
      </PageGrid>
    </Page>
  );
}
