import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { useAnalyticsPage } from '../features/analytics/useAnalyticsPage';

import { AnalyticsContent } from './analytics/AnalyticsContent';

export default function Analytics() {
  const state = useAnalyticsPage();

  if (!state.profileId) {
    return (
      <Page>
        <EmptyState message="Create a profile to view analytics." />
      </Page>
    );
  }

  if (state.isLoading) {
    return (
      <Page>
        <EmptyState message="Loading analytics…" />
      </Page>
    );
  }

  if (!state.summary) {
    return (
      <Page>
        <EmptyState message="No analytics data available." />
      </Page>
    );
  }

  return <AnalyticsContent state={state} />;
}
