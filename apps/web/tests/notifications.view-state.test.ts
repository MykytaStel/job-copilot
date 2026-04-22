import { describe, expect, it } from 'vitest';

import {
  countUnreadNotifications,
  resolveNotificationsViewState,
} from '../src/pages/notifications/viewState';

const baseNotification = {
  id: 'n1',
  profileId: 'profile-1',
  type: 'new_jobs_found' as const,
  title: 'New jobs found',
  createdAt: '2026-04-22T10:00:00Z',
};

describe('notifications view helpers', () => {
  it('counts unread notifications only', () => {
    expect(
      countUnreadNotifications([
        baseNotification,
        { ...baseNotification, id: 'n2', readAt: '2026-04-22T11:00:00Z' },
      ]),
    ).toBe(1);
  });

  it('resolves profile, loading, error, empty, and ready states', () => {
    expect(
      resolveNotificationsViewState({
        profileId: null,
        isLoading: false,
        error: null,
        notifications: [],
      }),
    ).toBe('missing_profile');

    expect(
      resolveNotificationsViewState({
        profileId: 'profile-1',
        isLoading: true,
        error: null,
        notifications: [],
      }),
    ).toBe('loading');

    expect(
      resolveNotificationsViewState({
        profileId: 'profile-1',
        isLoading: false,
        error: new Error('boom'),
        notifications: [],
      }),
    ).toBe('error');

    expect(
      resolveNotificationsViewState({
        profileId: 'profile-1',
        isLoading: false,
        error: null,
        notifications: [],
      }),
    ).toBe('empty');

    expect(
      resolveNotificationsViewState({
        profileId: 'profile-1',
        isLoading: false,
        error: null,
        notifications: [baseNotification],
      }),
    ).toBe('ready');
  });
});
