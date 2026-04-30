import { describe, expect, it, vi } from 'vitest';
import type { QueryClient } from '@tanstack/react-query';

import {
  invalidateApplicationSummaryQueries,
  invalidateFeedbackQueries,
  invalidateJobQueries,
  invalidateProfileAnalysisQueries,
} from '../src/lib/queryInvalidation';

function createQueryClientMock() {
  return {
    invalidateQueries: vi.fn().mockResolvedValue(undefined),
  } as unknown as QueryClient;
}

describe('query invalidation helpers', () => {
  it('keeps feedback invalidation focused on feedback queries', async () => {
    const queryClient = createQueryClientMock();

    await invalidateFeedbackQueries(queryClient, 'profile-1');

    expect(vi.mocked(queryClient.invalidateQueries).mock.calls).toEqual([
      [{ queryKey: ['feedback', 'profile-1'] }],
      [{ queryKey: ['feedback', 'stats', 'profile-1'] }],
      [{ queryKey: ['feedback', 'timeline', 'profile-1'] }],
    ]);
  });

  it('keeps job invalidation focused on job queries', async () => {
    const queryClient = createQueryClientMock();

    await invalidateJobQueries(queryClient, 'profile-1', 'job-1');

    expect(vi.mocked(queryClient.invalidateQueries).mock.calls).toEqual([
      [{ queryKey: ['jobs'] }],
      [{ queryKey: ['jobs', 'job-1', 'profile-1'] }],
    ]);
  });

  it('invalidates profile-analysis dependent ML queries', async () => {
    const queryClient = createQueryClientMock();

    await invalidateProfileAnalysisQueries(queryClient, 'profile-1');

    const calls = vi.mocked(queryClient.invalidateQueries).mock.calls;
    expect(calls).toContainEqual([{ queryKey: ['ml', 'rerank', 'profile-1'] }]);
    expect(calls).toContainEqual([{ queryKey: ['ml', 'fit', 'profile-1'] }]);
    expect(calls).toContainEqual([{ queryKey: ['ml', 'fitExplanation', 'profile-1'] }]);
    expect(calls).toContainEqual([{ queryKey: ['ml', 'coverLetter', 'profile-1'] }]);
    expect(calls).toContainEqual([{ queryKey: ['ml', 'interviewPrep', 'profile-1'] }]);
    expect(calls).toContainEqual([{ queryKey: ['analytics'] }]);
  });

  it('keeps application summary invalidation focused on application and dashboard queries', async () => {
    const queryClient = createQueryClientMock();

    await invalidateApplicationSummaryQueries(queryClient);

    expect(vi.mocked(queryClient.invalidateQueries).mock.calls).toEqual([
      [{ queryKey: ['applications'] }],
      [{ queryKey: ['dashboard', 'stats'] }],
    ]);
  });
});
