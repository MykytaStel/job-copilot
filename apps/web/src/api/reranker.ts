import { mlRequest } from './client';

export type BootstrapStatus = 'accepted' | 'running' | 'completed' | 'failed';

export const RERANKER_CACHE_INVALIDATION_EVENT =
  'job-copilot:reranker-cache-invalidation';

export type RerankerCacheInvalidationStatus = 'available' | 'degraded';

export type RerankerCacheInvalidationEventDetail = {
  profileId: string;
  status: RerankerCacheInvalidationStatus;
};

const rerankerCacheInvalidationStatusByProfile = new Map<
  string,
  RerankerCacheInvalidationStatus
>();

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

export function getRerankerCacheInvalidationStatus(
  profileId: string | null | undefined,
): RerankerCacheInvalidationStatus {
  if (!profileId) return 'available';
  return rerankerCacheInvalidationStatusByProfile.get(profileId) ?? 'available';
}

export async function invalidateRerankCache(profileId: string): Promise<boolean> {
  try {
    await mlRequest<void>('/api/v1/rerank/invalidate', {
      method: 'POST',
      body: JSON.stringify({ profile_id: profileId }),
    });
    emitRerankerCacheInvalidationStatus(profileId, 'available');
    return true;
  } catch {
    emitRerankerCacheInvalidationStatus(profileId, 'degraded');
    return false;
  }
}

function emitRerankerCacheInvalidationStatus(
  profileId: string,
  status: RerankerCacheInvalidationStatus,
) {
  rerankerCacheInvalidationStatusByProfile.set(profileId, status);

  if (typeof window === 'undefined') return;

  window.dispatchEvent(
    new CustomEvent<RerankerCacheInvalidationEventDetail>(RERANKER_CACHE_INVALIDATION_EVENT, {
      detail: { profileId, status },
    }),
  );
}
