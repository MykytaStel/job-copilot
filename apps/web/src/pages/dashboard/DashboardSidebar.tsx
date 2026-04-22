import { Link } from 'react-router-dom';
import { Bookmark, KanbanSquare, Sparkles, TrendingUp, XCircle, Zap } from 'lucide-react';
import type { DashboardPageState } from '../../features/dashboard/useDashboardPage';
import { STATUS_COLUMNS, STATUS_ICONS } from '../../features/dashboard/useDashboardPage';

import { AccentIconFrame } from '../../components/ui/AccentIconFrame';
import { AIInsightPanel } from '../../components/ui/AIInsightPanel';
import { Button } from '../../components/ui/Button';
import { Card, CardContent, CardHeader } from '../../components/ui/Card';

export function DashboardSidebar({
  insights,
  stats,
  applications,
  jobSummary,
}: Pick<DashboardPageState, 'insights' | 'stats' | 'applications' | 'jobSummary'>) {
  return (
    <>
      <AIInsightPanel insights={insights} />

      <Card className="border-border bg-card">
        <CardHeader className="gap-3">
          <div className="flex items-start gap-3">
            <AccentIconFrame size="md">
              <Sparkles className="h-4 w-4" />
            </AccentIconFrame>
            <div>
              <h2 className="m-0 text-base font-semibold text-card-foreground">Quick Actions</h2>
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
              <AccentIconFrame size="md">
                <XCircle className="h-4 w-4" />
              </AccentIconFrame>
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
                  <span className="font-medium text-card-foreground">{stats.byStatus[status] ?? 0}</span>
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
              <AccentIconFrame size="md">
                <Zap className="h-4 w-4" />
              </AccentIconFrame>
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
              <span className="font-medium text-muted-foreground">{jobSummary.inactiveJobs}</span>
            </div>
            <div className="flex items-center justify-between border-t border-border pt-3 text-sm">
              <span className="text-muted-foreground">Total tracked</span>
              <span className="font-medium text-card-foreground">{jobSummary.totalJobs}</span>
            </div>
          </CardContent>
        </Card>
      )}
    </>
  );
}
