import { Link } from 'react-router-dom';
import { ArrowRight, Sparkles, Zap } from 'lucide-react';
import type { DashboardPageState } from '../../features/dashboard/useDashboardPage';

import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { Card, CardContent } from '../../components/ui/Card';

import { buildDashboardViewModel } from './dashboard.view-model';

export function DashboardHero({
  jobSummary,
  allJobs,
  applications,
  topSource,
  stats,
  interviewedCount,
  mode,
}: Pick<
  DashboardPageState,
  'jobSummary' | 'allJobs' | 'applications' | 'topSource' | 'stats' | 'interviewedCount' | 'mode'
>) {
  const viewModel = buildDashboardViewModel({
    jobSummary,
    allJobs,
    applications,
    topSource,
    stats,
    interviewedCount,
    mode,
  });

  return (
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
                    {viewModel.modeLabel}
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
                  <div className="rounded-2xl border border-border/70 bg-white-a04 px-4 py-3">
                    <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                      Active jobs
                    </p>
                    <p className="m-0 mt-2 text-2xl font-bold text-card-foreground">
                      {viewModel.activeJobs}
                    </p>
                  </div>
                  <div className="rounded-2xl border border-border/70 bg-white-a04 px-4 py-3">
                    <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                      Tracked pipeline
                    </p>
                    <p className="m-0 mt-2 text-2xl font-bold text-card-foreground">
                      {viewModel.trackedPipeline}
                    </p>
                  </div>
                  <div className="rounded-2xl border border-border/70 bg-white-a04 px-4 py-3">
                    <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                      Source focus
                    </p>
                    <p className="m-0 mt-2 text-lg font-semibold text-card-foreground">
                      {viewModel.topSource}
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
  );
}
