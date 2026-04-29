import type { QueryClient } from '@tanstack/react-query';
import { queryKeys } from '../queryKeys';

export function invalidateApplicationSummaryQueries(queryClient: QueryClient) {
  return Promise.all([
    queryClient.invalidateQueries({ queryKey: queryKeys.applications.all() }),
    queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.stats() }),
  ]);
}

export function invalidateFeedbackViewQueries(queryClient: QueryClient, profileId?: string | null) {
  const tasks = [
    queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() }),
    queryClient.invalidateQueries({ queryKey: queryKeys.analytics.all() }),
  ];

  if (profileId) {
    tasks.push(queryClient.invalidateQueries({ queryKey: queryKeys.feedback.profile(profileId) }));
    tasks.push(queryClient.invalidateQueries({ queryKey: queryKeys.feedback.stats(profileId) }));
    tasks.push(queryClient.invalidateQueries({ queryKey: queryKeys.feedback.timeline(profileId) }));
    tasks.push(queryClient.invalidateQueries({ queryKey: queryKeys.ml.rerankPrefix(profileId) }));
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
