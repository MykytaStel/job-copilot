import { Bookmark, Briefcase, CalendarDays, Send } from 'lucide-react';
import type { DashboardPageState } from '../../features/dashboard/useDashboardPage';

import { StatCard } from '../../components/ui/StatCard';

import { buildDashboardViewModel } from './dashboard.view-model';

const icons = {
  briefcase: Briefcase,
  bookmark: Bookmark,
  send: Send,
  calendar: CalendarDays,
} as const;

export function DashboardStatsGrid({
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
    <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
      {viewModel.statCards.map((card) => {
        const Icon = icons[card.icon];
        return (
          <StatCard
            key={card.title}
            title={card.title}
            value={card.value}
            description={card.description}
            icon={Icon}
          />
        );
      })}
    </div>
  );
}
