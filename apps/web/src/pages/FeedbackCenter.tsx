import { Bookmark } from 'lucide-react';
import { Link } from 'react-router-dom';
import { Button } from '../components/ui/Button';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';

import { FeedbackCenterContent } from './feedback/FeedbackCenterContent';
import { useFeedbackCenterPage } from './feedback/useFeedbackCenterPage';

export default function FeedbackCenter() {
  const state = useFeedbackCenterPage();

  if (!state.profileId) {
    return (
      <Page>
        <PageHeader
          title="Feedback Center"
          description="Manage saved jobs, hidden roles, bad fits, and company preferences."
          breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Feedback' }]}
        />
        <div className="space-y-4">
          <EmptyState
            icon={<Bookmark className="h-5 w-5" />}
            message="Feedback needs an active profile"
            description="Create or load a profile first so saved jobs, hidden jobs, and company preferences stay scoped to the right candidate context."
          />
          <Link to="/profile" className="inline-flex no-underline">
            <Button>Open Profile &amp; Search</Button>
          </Link>
        </div>
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
