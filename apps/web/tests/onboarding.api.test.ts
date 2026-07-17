import { afterEach, describe, expect, it, vi } from 'vitest';

import { advanceProfileOnboarding, getProfileOnboarding } from '../src/api/onboarding';

describe('profile onboarding api', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it('loads and advances profile-scoped onboarding state', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          profile_id: 'profile-1',
          current_step: 'cv',
          completed_steps: [],
          skipped_steps: [],
          completed_at: null,
          updated_at: '2026-07-17T00:00:00Z',
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({
          profile_id: 'profile-1',
          current_step: 'profile',
          completed_steps: ['cv'],
          skipped_steps: [],
          completed_at: null,
          updated_at: '2026-07-17T00:01:00Z',
        }),
      });
    vi.stubGlobal('fetch', fetchMock);

    const initial = await getProfileOnboarding('profile-1');
    const advanced = await advanceProfileOnboarding('profile-1', 'cv');

    expect(initial.current_step).toBe('cv');
    expect(advanced.current_step).toBe('profile');
    expect(String(fetchMock.mock.calls[0]?.[0])).toContain(
      '/api/v1/profiles/profile-1/onboarding',
    );
    expect(fetchMock.mock.calls[1]?.[1]).toEqual(
      expect.objectContaining({ method: 'PATCH' }),
    );
    expect(JSON.parse(fetchMock.mock.calls[1]![1]!.body as string)).toEqual({
      step: 'cv',
      outcome: 'completed',
    });
  });
});
