import { CalendarClock, MapPin } from 'lucide-react';
import type { JobDetailsPageState } from '../../features/job-details/useJobDetailsPage';

import { HeroMetric, Section, formatDate } from './components';

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
        <HeroMetric label="First seen" value={formatDate(job.firstSeenAt) ?? 'n/a'} icon={CalendarClock} />
        <HeroMetric label="Last seen" value={formatDate(job.lastSeenAt) ?? 'n/a'} icon={CalendarClock} />
        <HeroMetric
          label="Inactive at"
          value={formatDate(job.inactivatedAt) ?? 'n/a'}
          icon={CalendarClock}
        />
        <HeroMetric
          label="Reactivated"
          value={formatDate(job.reactivatedAt) ?? 'n/a'}
          icon={CalendarClock}
        />
        <HeroMetric label="Source id" value={job.primaryVariant?.sourceJobId ?? 'n/a'} icon={MapPin} />
      </div>
    </Section>
  );
}
