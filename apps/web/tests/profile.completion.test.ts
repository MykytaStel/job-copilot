import { describe, expect, it } from 'vitest';

import { buildProfileCompletionState } from '../src/features/profile/profileCompletion';

describe('buildProfileCompletionState', () => {
  it('counts completed checkpoints and reports missing labels', () => {
    const state = buildProfileCompletionState({
      name: 'Jane Doe',
      email: 'jane@example.com',
      location: '',
      rawText: 'Senior frontend engineer',
      yearsOfExperience: '',
      salaryMin: '4000',
      salaryMax: '',
      salaryCurrency: 'USD',
      languages: [],
      analysisReady: false,
    });

    expect(state.completed).toBe(3);
    expect(state.total).toBe(8);
    expect(state.percent).toBe(38);
    expect(state.missingLabels).toEqual([
      'Location',
      'Experience',
      'Compensation',
      'Languages',
      'Analysis',
    ]);
  });

  it('marks compensation complete only when range and currency are present', () => {
    const state = buildProfileCompletionState({
      name: 'Jane Doe',
      email: 'jane@example.com',
      location: 'Kyiv',
      rawText: 'Senior frontend engineer',
      yearsOfExperience: '7',
      salaryMin: '4000',
      salaryMax: '5500',
      salaryCurrency: 'USD',
      languages: ['english'],
      analysisReady: true,
    });

    expect(state.percent).toBe(100);
    expect(state.missingLabels).toEqual([]);
  });
});
