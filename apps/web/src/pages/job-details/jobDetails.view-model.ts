import type { JobDetailsPageState } from '../../features/job-details/useJobDetailsPage';

import { formatSalary } from './components';

export function buildJobDetailsViewModel({
  job,
  fit,
}: Pick<JobDetailsPageState, 'job' | 'fit'>) {
  const salary = formatSalary(job?.salaryMin, job?.salaryMax, job?.salaryCurrency);
  const sourceLabel = job?.primaryVariant?.source
    ? job.primaryVariant.source.replace('_', '.')
    : 'Unknown source';
  const descriptionQuality = fit?.descriptionQuality ?? job?.presentation?.descriptionQuality;
  const topBadges = [sourceLabel, job?.seniority ?? null, job?.remoteType ?? null].filter(
    Boolean,
  ) as string[];
  const skillBadges = [...(job?.presentation?.badges ?? []), ...(fit?.matchedTerms ?? [])].slice(
    0,
    10,
  );
  const lifecycleStatus = job?.lifecycleStage ?? (job?.isActive === false ? 'inactive' : 'active');

  return {
    salary,
    sourceLabel,
    descriptionQuality,
    topBadges,
    skillBadges,
    lifecycleStatus,
  };
}
