import { logUserEvent } from '../../api';

const SESSION_STORAGE_KEY = 'job-copilot.job-impressions.v1';
const runtimeLoggedImpressions = new Set<string>();

export type JobImpressionSurface =
  | 'dashboard_recent_jobs'
  | 'ranked_search_results';

function readLoggedImpressionKeys() {
  if (typeof window === 'undefined') {
    return new Set<string>();
  }

  try {
    const raw = window.sessionStorage.getItem(SESSION_STORAGE_KEY);
    if (!raw) {
      return new Set<string>();
    }

    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return new Set<string>();
    }

    return new Set(
      parsed.filter((value): value is string => typeof value === 'string'),
    );
  } catch {
    return new Set<string>();
  }
}

function persistLoggedImpressionKeys(keys: Set<string>) {
  if (typeof window === 'undefined') {
    return;
  }

  try {
    window.sessionStorage.setItem(
      SESSION_STORAGE_KEY,
      JSON.stringify(Array.from(keys).slice(-5000)),
    );
  } catch {
    // Ignore storage failures. The runtime set still suppresses duplicate spam.
  }
}

function impressionKey(
  profileId: string,
  surface: JobImpressionSurface,
  jobId: string,
) {
  return `${profileId}:${surface}:${jobId}`;
}

export async function logJobImpressionsOnce({
  profileId,
  jobs,
  surface,
}: {
  profileId: string | null | undefined;
  jobs: Array<{ id: string }>;
  surface: JobImpressionSurface;
}): Promise<void> {
  if (!profileId || jobs.length === 0) {
    return;
  }

  const sessionLoggedKeys = readLoggedImpressionKeys();
  const seenJobIds = new Set<string>();
  const pendingJobIds: string[] = [];

  for (const job of jobs) {
    if (!job.id || seenJobIds.has(job.id)) {
      continue;
    }
    seenJobIds.add(job.id);

    const key = impressionKey(profileId, surface, job.id);
    if (runtimeLoggedImpressions.has(key) || sessionLoggedKeys.has(key)) {
      continue;
    }

    runtimeLoggedImpressions.add(key);
    sessionLoggedKeys.add(key);
    pendingJobIds.push(job.id);
  }

  if (pendingJobIds.length === 0) {
    return;
  }

  persistLoggedImpressionKeys(sessionLoggedKeys);

  await Promise.allSettled(
    pendingJobIds.map((jobId) =>
      logUserEvent(profileId, {
        eventType: 'job_impression',
        jobId,
        payloadJson: {
          surface,
        },
      }),
    ),
  );
}
