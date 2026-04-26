import { useCallback, useEffect, useRef, useState } from 'react';

import { bootstrapReranker, getBootstrapStatus } from '../../api/reranker';

export type RetrainStatus = 'idle' | 'running' | 'done' | 'error';

export function useRetrainModel(profileId: string | null) {
  const [status, setStatus] = useState<RetrainStatus>('idle');
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const stopPolling = useCallback(() => {
    if (pollingRef.current !== null) {
      clearInterval(pollingRef.current);
      pollingRef.current = null;
    }
  }, []);

  useEffect(() => () => stopPolling(), [stopPolling]);

  const pollStatus = useCallback(
    async (taskId: string) => {
      try {
        const task = await getBootstrapStatus(taskId);
        if (task.status === 'completed') {
          stopPolling();
          setStatus('done');
        } else if (task.status === 'failed') {
          stopPolling();
          setStatus('error');
          setErrorMsg(task.error ?? 'Retraining failed');
        }
        // 'accepted' | 'running' → keep polling
      } catch {
        stopPolling();
        setStatus('error');
        setErrorMsg('Failed to fetch task status');
      }
    },
    [stopPolling],
  );

  const trigger = useCallback(async () => {
    if (!profileId || status === 'running') return;
    setStatus('running');
    setErrorMsg(null);
    try {
      const accepted = await bootstrapReranker(profileId);
      pollingRef.current = setInterval(() => {
        void pollStatus(accepted.task_id);
      }, 3000);
    } catch (err) {
      setStatus('error');
      setErrorMsg(err instanceof Error ? err.message : 'Failed to start retraining');
    }
  }, [profileId, status, pollStatus]);

  return { trigger, status, errorMsg };
}
