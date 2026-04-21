import type { DashboardPageState } from '../../features/dashboard/useDashboardPage';

export function buildDashboardViewModel({
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
  return {
    activeJobs: jobSummary?.activeJobs ?? allJobs.length,
    trackedPipeline: applications.length,
    topSource,
    modeLabel: mode === 'ranked' ? 'Ranked mode' : 'Recent mode',
    statCards: [
      {
        title: 'Активних вакансій',
        value: jobSummary?.activeJobs ?? allJobs.length,
        description: 'зараз у базі',
        icon: 'briefcase' as const,
      },
      {
        title: 'Збережено',
        value: stats?.byStatus.saved ?? 0,
        description: 'у pipeline',
        icon: 'bookmark' as const,
      },
      {
        title: 'Подано заявки',
        value: stats?.byStatus.applied ?? 0,
        description: 'готові до follow-up',
        icon: 'send' as const,
      },
      {
        title: "Інтерв'ю",
        value: interviewedCount,
        description: 'interview + offer',
        icon: 'calendar' as const,
      },
    ],
  };
}
