import { Page, PageGrid } from '../../components/ui/Page';
import type { JobDetailsPageState } from '../../features/job-details/useJobDetailsPage';

import { JobDetailsAiTab } from './JobDetailsAiTab';
import { JobDetailsLifecycleTab } from './JobDetailsLifecycleTab';
import { JobDetailsHeader } from './JobDetailsHeader';
import { JobDetailsMatchTab } from './JobDetailsMatchTab';
import { JobDetailsOverviewTab } from './JobDetailsOverviewTab';
import { JobDetailsSidebar } from './JobDetailsSidebar';
import { JobDetailsTabs } from './JobDetailsTabs';
import { buildJobDetailsViewModel } from './jobDetails.view-model';

export function JobDetailsContent({ state }: { state: JobDetailsPageState }) {
  const { job, activeTab, setActiveTab, profileId, deterministicFit, fitExplanation, fitExplanationLoading, coverLetter, coverLetterLoading, interviewPrep, interviewPrepLoading, setGenerateCoverLetter, setGenerateInterviewPrep, fit } =
    state;

  if (!job) {
    return null;
  }

  const viewModel = buildJobDetailsViewModel(state);

  return (
    <Page>
      <JobDetailsHeader
        state={state}
        salary={viewModel.salary}
        sourceLabel={viewModel.sourceLabel}
        descriptionQuality={viewModel.descriptionQuality}
        topBadges={viewModel.topBadges}
        lifecycleStatus={viewModel.lifecycleStatus}
      />

      <PageGrid aside={<JobDetailsSidebar state={state} />}>
        <div className="space-y-6">
          <JobDetailsTabs activeTab={activeTab} setActiveTab={setActiveTab} />

          {activeTab === 'overview' ? (
            <JobDetailsOverviewTab
              job={job}
              descriptionQuality={viewModel.descriptionQuality}
              skillBadges={viewModel.skillBadges}
            />
          ) : null}

          {activeTab === 'match' ? <JobDetailsMatchTab fit={fit} /> : null}

          {activeTab === 'ai' ? (
            <JobDetailsAiTab
              profileId={profileId}
              deterministicFit={deterministicFit}
              fitExplanation={fitExplanation}
              fitExplanationLoading={fitExplanationLoading}
              coverLetter={coverLetter}
              coverLetterLoading={coverLetterLoading}
              interviewPrep={interviewPrep}
              interviewPrepLoading={interviewPrepLoading}
              setGenerateCoverLetter={setGenerateCoverLetter}
              setGenerateInterviewPrep={setGenerateInterviewPrep}
            />
          ) : null}

          {activeTab === 'lifecycle' ? <JobDetailsLifecycleTab job={job} /> : null}
        </div>
      </PageGrid>
    </Page>
  );
}
