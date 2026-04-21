import { Page, PageGrid } from '../../components/ui/Page';
import type { DashboardPageState } from '../../features/dashboard/useDashboardPage';

import { DashboardHero } from './DashboardHero';
import { DashboardMatchesSection } from './DashboardMatchesSection';
import { DashboardSidebar } from './DashboardSidebar';
import { DashboardStatsGrid } from './DashboardStatsGrid';

export function DashboardContent({ state }: { state: DashboardPageState }) {
  return (
    <Page>
      <DashboardHero
        jobSummary={state.jobSummary}
        allJobs={state.allJobs}
        applications={state.applications}
        topSource={state.topSource}
        stats={state.stats}
        interviewedCount={state.interviewedCount}
        mode={state.mode}
      />

      {state.error && <p className="error">{state.error}</p>}

      <DashboardStatsGrid
        jobSummary={state.jobSummary}
        allJobs={state.allJobs}
        applications={state.applications}
        topSource={state.topSource}
        stats={state.stats}
        interviewedCount={state.interviewedCount}
        mode={state.mode}
      />

      <PageGrid
        aside={
          <DashboardSidebar
            insights={state.insights}
            stats={state.stats}
            applications={state.applications}
            jobSummary={state.jobSummary}
          />
        }
      >
        <DashboardMatchesSection
          mode={state.mode}
          setSortByScore={state.setSortByScore}
          search={state.search}
          setSearch={state.setSearch}
          jobs={state.jobs}
          allJobs={state.allJobs}
          jobsLoading={state.jobsLoading}
          rankData={state.rankData}
          lifecycleOptions={state.lifecycleOptions}
          selectedLifecycle={state.selectedLifecycle}
          updateFilters={state.updateFilters}
          sourceOptions={state.sourceOptions}
          selectedSource={state.selectedSource}
          sourcesError={state.sourcesError}
          applicationByJob={state.applicationByJob}
          scoreById={state.scoreById}
          saveMutation={state.saveMutation}
          hideMutation={state.hideMutation}
          badFitMutation={state.badFitMutation}
          unmarkBadFitMutation={state.unmarkBadFitMutation}
        />
      </PageGrid>
    </Page>
  );
}
