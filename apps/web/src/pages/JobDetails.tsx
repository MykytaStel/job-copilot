import { SkeletonPage } from '../components/Skeleton';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { useJobDetailsPage } from '../features/job-details/useJobDetailsPage';

import { JobDetailsContent } from './job-details/JobDetailsContent';

export default function JobDetails() {
  const state = useJobDetailsPage();

  if (state.isLoading) {
    return <SkeletonPage />;
  }

  if (!state.job) {
    return (
      <Page>
        <EmptyState
          message={state.error instanceof Error ? state.error.message : 'Вакансія не знайдена'}
        />
      </Page>
    );
  }

  return <JobDetailsContent state={state} />;
}
