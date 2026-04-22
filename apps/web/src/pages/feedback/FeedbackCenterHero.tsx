import { Badge } from '../../components/ui/Badge';
import { Card, CardContent } from '../../components/ui/Card';
import type { FeedbackCenterPageState } from './useFeedbackCenterPage';

export function FeedbackCenterHero({
  savedJobs,
  badFitJobs,
  tabCounts,
}: Pick<FeedbackCenterPageState, 'savedJobs' | 'badFitJobs' | 'tabCounts'>) {
  return (
    <Card className="overflow-hidden border-border bg-card">
      <CardContent className="p-0">
        <div className="relative">
          <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-primary/8 via-accent/5 to-transparent" />
          <div className="relative flex flex-col gap-6 p-7 lg:flex-row lg:items-end lg:justify-between">
            <div className="max-w-3xl space-y-3">
              <div className="flex flex-wrap gap-2">
                <Badge
                  variant="default"
                  className="border-0 bg-primary/15 px-2 py-0.5 text-xs text-primary"
                >
                  Feedback loops
                </Badge>
                <Badge
                  variant="muted"
                  className="px-2 py-0.5 text-[10px] uppercase tracking-wide"
                >
                  Saved, hidden, bad-fit, companies
                </Badge>
              </div>
              <h2 className="m-0 text-2xl font-bold text-card-foreground lg:text-3xl">
                Train the ranking engine with explicit feedback
              </h2>
              <p className="m-0 text-sm leading-7 text-muted-foreground lg:text-base">
                Saved jobs, hidden roles, bad fits, and company allow/block lists directly shape
                what rises or disappears from the feed.
              </p>
            </div>
            <div className="grid gap-3 sm:grid-cols-3 lg:min-w-[420px]">
              <div className="rounded-2xl border border-border/70 bg-white-a04 px-4 py-3">
                <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                  Saved
                </p>
                <p className="m-0 mt-2 text-2xl font-bold text-card-foreground">
                  {savedJobs.length}
                </p>
              </div>
              <div className="rounded-2xl border border-border/70 bg-white-a04 px-4 py-3">
                <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                  Bad fit
                </p>
                <p className="m-0 mt-2 text-2xl font-bold text-card-foreground">
                  {badFitJobs.length}
                </p>
              </div>
              <div className="rounded-2xl border border-border/70 bg-white-a04 px-4 py-3">
                <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                  Company lists
                </p>
                <p className="m-0 mt-2 text-2xl font-bold text-card-foreground">
                  {tabCounts.companies}
                </p>
              </div>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
