import type { JobPosting } from '@job-copilot/shared';

function pushUnique(target: string[], value?: string | null) {
  const normalized = value?.trim();
  if (!normalized || target.includes(normalized)) {
    return;
  }

  target.push(normalized);
}

export function getJobLifecycleLabels(job: Pick<JobPosting, 'presentation'>): string[] {
  const labels: string[] = [];
  pushUnique(
    labels,
    job.presentation?.lifecyclePrimaryLabel ?? job.presentation?.freshnessLabel ?? undefined,
  );
  pushUnique(labels, job.presentation?.lifecycleSecondaryLabel);
  return labels;
}

export function getJobMetaLabels(job: Pick<JobPosting, 'presentation'>): string[] {
  const labels: string[] = [];
  pushUnique(labels, job.presentation?.locationLabel);
  pushUnique(labels, job.presentation?.workModeLabel);
  pushUnique(labels, job.presentation?.salaryLabel);

  for (const label of getJobLifecycleLabels(job)) {
    pushUnique(labels, label);
  }

  return labels;
}

export const ACTIVE_ONLY_EMPTY_STATE_MESSAGE =
  'No active jobs matched this search profile. Inactive or older lifecycle history still remains available outside ranked search.';
