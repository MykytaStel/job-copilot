import { mlRequest } from './client';

export type BootstrapStatus = 'accepted' | 'running' | 'completed' | 'failed';

export type BootstrapTaskAccepted = {
  task_id: string;
  status: 'accepted';
};

export type BootstrapTaskStatus = {
  task_id: string;
  profile_id: string | null;
  status: BootstrapStatus;
  error: string | null;
  started_at: string | null;
  finished_at: string | null;
};

export function bootstrapReranker(
  profileId: string,
  minExamples = 30,
): Promise<BootstrapTaskAccepted> {
  return mlRequest<BootstrapTaskAccepted>('/api/v1/reranker/bootstrap', {
    method: 'POST',
    body: JSON.stringify({ profile_id: profileId, min_examples: minExamples }),
  });
}

export function getBootstrapStatus(taskId: string): Promise<BootstrapTaskStatus> {
  return mlRequest<BootstrapTaskStatus>(`/api/v1/reranker/bootstrap/${taskId}`);
}
