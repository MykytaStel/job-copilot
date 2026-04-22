import { describe, expect, it } from 'vitest';

import type { JobPosting } from '@job-copilot/shared';
import {
  ACTIVE_ONLY_EMPTY_STATE_MESSAGE,
  getJobLifecycleLabels,
  getJobMetaLabels,
} from '../src/lib/jobPresentation';

function sampleJob(presentation: JobPosting['presentation']): Pick<JobPosting, 'presentation'> {
  return { presentation };
}

describe('job presentation helpers', () => {
  it('returns both lifecycle labels for posted multi-day active jobs', () => {
    expect(
      getJobLifecycleLabels(
        sampleJob({
          title: 'Frontend Engineer',
          company: 'Acme',
          badges: [],
          freshnessLabel: 'Posted 2026-04-15',
          lifecyclePrimaryLabel: 'Posted 2026-04-15',
          lifecycleSecondaryLabel: 'Last confirmed active 2026-04-22',
        }),
      ),
    ).toEqual(['Posted 2026-04-15', 'Last confirmed active 2026-04-22']);
  });

  it('falls back to seen since when the source has no absolute posted date', () => {
    expect(
      getJobMetaLabels(
        sampleJob({
          title: 'Frontend Engineer',
          company: 'Acme',
          badges: [],
          freshnessLabel: 'Seen since 2026-04-15',
          lifecyclePrimaryLabel: 'Seen since 2026-04-15',
          lifecycleSecondaryLabel: 'Last confirmed active 2026-04-22',
        }),
      ),
    ).toContain('Seen since 2026-04-15');
  });

  it('keeps ranked-search empty copy explicit about active-only scope', () => {
    expect(ACTIVE_ONLY_EMPTY_STATE_MESSAGE).toBe(
      'No active jobs matched this search profile. Inactive or older lifecycle history still remains available outside ranked search.',
    );
  });
});
