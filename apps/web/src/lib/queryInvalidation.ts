import type { QueryClient } from '@tanstack/react-query';
import { queryKeys } from '../queryKeys';

export function invalidateApplicationSummaryQueries(queryClient: QueryClient) {
  return Promise.all([
    queryClient.invalidateQueries({ queryKey: queryKeys.applications.all() }),
    queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.stats() }),
  ]);
}

export function invalidateFeedbackQueries(queryClient: QueryClient, profileId?: string | null) {
  if (!profileId) {
    return Promise.resolve([]);
  }

  return Promise.all([
    queryClient.invalidateQueries({ queryKey: queryKeys.feedback.profile(profileId) }),
    queryClient.invalidateQueries({ queryKey: queryKeys.feedback.stats(profileId) }),
    queryClient.invalidateQueries({ queryKey: queryKeys.feedback.timeline(profileId) }),
  ]);
}

export function invalidateJobQueries(
  queryClient: QueryClient,
  profileId?: string | null,
  jobId?: string | null,
) {
  const tasks = [queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() })];

  if (profileId && jobId) {
    tasks.push(queryClient.invalidateQueries({ queryKey: queryKeys.jobs.detail(jobId, profileId) }));
  }

  return Promise.all(tasks);
}

export function invalidateJobAiQueries(
  queryClient: QueryClient,
  profileId?: string | null,
  jobId?: string | null,
) {
  if (!profileId || !jobId) {
    return Promise.resolve([]);
  }

  return Promise.all([
    queryClient.invalidateQueries({ queryKey: queryKeys.ml.fit(profileId, jobId) }),
    queryClient.invalidateQueries({ queryKey: queryKeys.ml.fitExplanation(profileId, jobId) }),
    queryClient.invalidateQueries({ queryKey: queryKeys.ml.coverLetter(profileId, jobId) }),
    queryClient.invalidateQueries({ queryKey: queryKeys.ml.interviewPrep(profileId, jobId) }),
  ]);
}

export function invalidateProfileAnalysisQueries(queryClient: QueryClient, profileId: string) {
  return Promise.all([
    queryClient.invalidateQueries({ queryKey: queryKeys.ml.rerankPrefix(profileId) }),
    queryClient.invalidateQueries({ queryKey: queryKeys.ml.fitPrefix(profileId) }),
    queryClient.invalidateQueries({ queryKey: queryKeys.ml.fitExplanationPrefix(profileId) }),
    queryClient.invalidateQueries({ queryKey: queryKeys.ml.coverLetterPrefix(profileId) }),
    queryClient.invalidateQueries({ queryKey: queryKeys.ml.interviewPrepPrefix(profileId) }),
    queryClient.invalidateQueries({ queryKey: queryKeys.analytics.all() }),
  ]);
}
