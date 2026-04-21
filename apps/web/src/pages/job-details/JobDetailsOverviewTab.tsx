import { Briefcase } from 'lucide-react';
import type { JobDetailsPageState } from '../../features/job-details/useJobDetailsPage';

import { Badge } from '../../components/ui/Badge';
import { Section } from './components';

export function JobDetailsOverviewTab({
  job,
  descriptionQuality,
  skillBadges,
}: {
  job: NonNullable<JobDetailsPageState['job']>;
  descriptionQuality: string | undefined;
  skillBadges: string[];
}) {
  return (
    <Section
      title="Role Overview"
      description="Read the job description and the strongest structured signals extracted from the posting."
      icon={Briefcase}
    >
      <div className="space-y-5">
        <div className="rounded-2xl border border-border/70 bg-white/[0.03] p-4">
          <p className="m-0 text-sm font-semibold text-card-foreground">Job description</p>
          <div className="mt-4 whitespace-pre-wrap text-sm leading-7 text-muted-foreground">
            {job.description || 'No description available.'}
          </div>
        </div>

        {descriptionQuality === 'weak' ? (
          <div className="rounded-2xl border border-content-warning/40 bg-content-warning/10 p-4">
            <p className="m-0 text-sm font-semibold text-card-foreground">
              Description quality warning
            </p>
            <p className="m-0 mt-3 text-sm leading-7 text-muted-foreground">
              This vacancy looks partially extracted or too short. Treat the fit score as
              lower-confidence until the source page or a richer reparse confirms the missing
              context.
            </p>
          </div>
        ) : null}

        {skillBadges.length > 0 ? (
          <div className="rounded-2xl border border-border/70 bg-white/[0.03] p-4">
            <p className="m-0 text-sm font-semibold text-card-foreground">Skills and signals</p>
            <div className="mt-4 flex flex-wrap gap-2">
              {skillBadges.map((item) => (
                <Badge key={item} variant="muted" className="px-3 py-1 text-xs">
                  {item}
                </Badge>
              ))}
            </div>
          </div>
        ) : null}
      </div>
    </Section>
  );
}
