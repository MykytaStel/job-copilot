import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';

import { FeedbackCenterContent } from './feedback/FeedbackCenterContent';
import { useFeedbackCenterPage } from './feedback/useFeedbackCenterPage';

export default function FeedbackCenter() {
  const state = useFeedbackCenterPage();

  if (!state.profileId) {
    return (
      <Page>
        <EmptyState message="Create a profile to view feedback." />
      </Page>
    );
  }

  if (state.isLoading) {
    return (
      <Page>
        <EmptyState message="Loading feedback…" />
      </Page>
    );
  }

  return <FeedbackCenterContent state={state} />;
}
