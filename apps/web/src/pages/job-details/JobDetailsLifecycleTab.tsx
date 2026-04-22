import { CalendarClock, MapPin } from 'lucide-react';
import type { JobDetailsPageState } from '../../features/job-details/useJobDetailsPage';
import { formatOptionalDate } from '../../lib/format';

import { HeroMetric, Section } from './components';

export function JobDetailsLifecycleTab({
  job,
}: {
  job: NonNullable<JobDetailsPageState['job']>;
}) {
  return (
    <Section
      title="Lifecycle Metadata"
      description="Timeline and source-specific identifiers from the canonical job record."
      icon={CalendarClock}
    >
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        <HeroMetric
          label="Posted at"
          value={formatOptionalDate(job.postedAt) ?? 'n/a'}
          icon={CalendarClock}
        />
        <HeroMetric
          label="First seen"
          value={formatOptionalDate(job.firstSeenAt) ?? 'n/a'}
          icon={CalendarClock}
        />
        <HeroMetric
          label="Last seen"
          value={formatOptionalDate(job.lastSeenAt) ?? 'n/a'}
          icon={CalendarClock}
        />
        <HeroMetric
          label="Inactive at"
          value={formatOptionalDate(job.inactivatedAt) ?? 'n/a'}
          icon={CalendarClock}
        />
        <HeroMetric
          label="Reactivated"
          value={formatOptionalDate(job.reactivatedAt) ?? 'n/a'}
          icon={CalendarClock}
        />
        <HeroMetric label="Source id" value={job.primaryVariant?.sourceJobId ?? 'n/a'} icon={MapPin} />
      </div>
    </Section>
  );
}
