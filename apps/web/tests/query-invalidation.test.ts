import { describe, expect, it, vi } from 'vitest';
import type { QueryClient } from '@tanstack/react-query';

import {
  invalidateApplicationSummaryQueries,
  invalidateFeedbackViewQueries,
  invalidateProfileAnalysisQueries,
} from '../src/lib/queryInvalidation';

function createQueryClientMock() {
  return {
    invalidateQueries: vi.fn().mockResolvedValue(undefined),
  } as unknown as QueryClient;
}

describe('query invalidation helpers', () => {
  it('invalidates rerank and analytics queries after feedback mutations', async () => {
    const queryClient = createQueryClientMock();

    await invalidateFeedbackViewQueries(queryClient, 'profile-1');

    const calls = vi.mocked(queryClient.invalidateQueries).mock.calls;
    expect(calls).toContainEqual([{ queryKey: ['jobs'] }]);
    expect(calls).toContainEqual([{ queryKey: ['analytics'] }]);
    expect(calls).toContainEqual([{ queryKey: ['feedback', 'profile-1'] }]);
    expect(calls).toContainEqual([{ queryKey: ['ml', 'rerank', 'profile-1'] }]);
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
