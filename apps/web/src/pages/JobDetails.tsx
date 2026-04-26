import { JobDetailsSkeleton } from '../components/JobDetailsSkeleton';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { useJobDetailsPage } from '../features/job-details/useJobDetailsPage';
import { JobDetailsContent } from './job-details/JobDetailsContent';

export default function JobDetails() {
  const state = useJobDetailsPage();

  if (state.isLoading) {
    return <JobDetailsSkeleton />;
  }

  if (!state.job) {
    return (
      <Page>
        <EmptyState
          message="Job not found"
          description={
            state.error instanceof Error
              ? state.error.message
              : 'This job may have been removed or is not available anymore.'
          }
        />
      </Page>
    );
  }

  return <JobDetailsContent state={state} />;
}
