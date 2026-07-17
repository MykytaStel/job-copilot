import { useEffect, useMemo, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useLocation, useNavigate } from 'react-router-dom';

import { getMlReady, isMlDegraded } from '../api/ml-health';
import { getUnreadCount } from '../api/notifications';
import { getProfileOnboarding } from '../api/onboarding';
import { getProfile } from '../api/profiles';
import { getSourceHealth } from '../api/source-health';
import { hasToken } from '../lib/authSession';
import { readProfileId } from '../lib/profileSession';
import { queryKeys } from '../queryKeys';
import { navigation } from './navigation';

export function useAppShell() {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [mobileNavOpen, setMobileNavOpen] = useState(false);
  const [mlBannerDismissed, setMlBannerDismissed] = useState(false);
  const location = useLocation();
  const navigate = useNavigate();
  const hasSession = hasToken() && !!readProfileId();

  const { data: profile, isLoading: profileLoading } = useQuery({
    queryKey: queryKeys.profile.root(),
    queryFn: getProfile,
    enabled: hasSession,
  });

  const { data: onboarding, isLoading: onboardingLoading } = useQuery({
    queryKey: queryKeys.onboarding.profile(profile?.id ?? 'none'),
    queryFn: () => getProfileOnboarding(profile!.id),
    enabled: !!profile?.id,
  });

  useEffect(() => {
    if (!profileLoading && !profile && !hasSession) {
      navigate('/auth', { replace: true });
      return;
    }
    if (
      profile &&
      !onboardingLoading &&
      onboarding?.current_step !== 'complete' &&
      location.pathname !== '/setup'
    ) {
      navigate('/setup', { replace: true });
    }
  }, [profile, profileLoading, onboarding, onboardingLoading, hasSession, navigate, location.pathname]);

  const { data: mlReady } = useQuery({
    queryKey: queryKeys.ml.ready(),
    queryFn: getMlReady,
    refetchInterval: 60_000,
    retry: false,
    staleTime: 30_000,
  });

  const mlDegraded = mlReady ? isMlDegraded(mlReady) : false;

  const { data: sourceHealth = [] } = useQuery({
    queryKey: queryKeys.sources.health(),
    queryFn: getSourceHealth,
    enabled: !!profile?.id,
    refetchInterval: 60_000,
    retry: false,
    staleTime: 30_000,
  });
  const degradedSourceCount = sourceHealth.filter((source) => source.degraded).length;

  const { data: unreadCount = 0 } = useQuery({
    queryKey: queryKeys.notifications.unreadCount(profile?.id ?? 'none'),
    queryFn: () => getUnreadCount(),
    enabled: !!profile?.id,
    staleTime: 30_000,
  });

  const activeNavItem = useMemo(() => {
    return (
      navigation.find((item) => {
        if (item.end) {
          return location.pathname === item.to;
        }

        return location.pathname.startsWith(item.to);
      }) ?? null
    );
  }, [location.pathname]);

  return {
    sidebarCollapsed,
    setSidebarCollapsed,
    mobileNavOpen,
    setMobileNavOpen,
    location,
    profile,
    profileLoading,
    unreadCount,
    degradedSourceCount,
    activeNavItem,
    mlDegraded: mlDegraded && !mlBannerDismissed,
    dismissMlBanner: () => setMlBannerDismissed(true),
  };
}

export type AppShellState = ReturnType<typeof useAppShell>;
