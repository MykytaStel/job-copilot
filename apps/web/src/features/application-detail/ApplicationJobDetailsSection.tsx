import { BriefcaseBusiness } from 'lucide-react';
import type { ApplicationDetail } from '@job-copilot/shared';

import { formatDate } from '../../lib/format';
import { DescriptionBlock } from './ApplicationDetailCards';
import { InnerPanel, Panel } from './ApplicationDetailLayout';

export function JobDetailsSection({ detail }: { detail: ApplicationDetail }) {
  const { job } = detail;

  return (
    <Panel
      title="Role Snapshot"
      description="Reference copy of the linked job posting and its source metadata."
      icon={BriefcaseBusiness}
    >
      <div className="grid gap-4 md:grid-cols-2">
        {job.url ? (
          <InnerPanel title="Source link" description="Original posting used for the application.">
            <a
              href={job.url}
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm text-primary no-underline hover:underline"
            >
              {job.url}
            </a>
          </InnerPanel>
        ) : null}

        <InnerPanel
          title="Posting timeline"
          description="Current metadata from the linked job record."
        >
          <div className="space-y-2 text-sm text-muted-foreground">
            <div className="flex items-center justify-between gap-3">
              <span>Created</span>
              <span className="font-medium text-card-foreground">
                {formatDate(job.createdAt) ?? 'n/a'}
              </span>
            </div>
          </div>
        </InnerPanel>
      </div>

      <DescriptionBlock text={job.description || 'No description available.'} />
    </Panel>
  );
}
