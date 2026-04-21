import { useMemo, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useLocation } from 'react-router-dom';

import { getUnreadCount } from '../api/notifications';
import { getProfile } from '../api/profiles';
import { queryKeys } from '../queryKeys';
import { navigation } from './navigation';

export function useAppShell() {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [mobileNavOpen, setMobileNavOpen] = useState(false);
  const location = useLocation();

  const { data: profile, isLoading: profileLoading } = useQuery({
    queryKey: queryKeys.profile.root(),
    queryFn: getProfile,
  });

  const { data: unreadCount = 0 } = useQuery({
    queryKey: queryKeys.notifications.unreadCount(profile?.id ?? 'none'),
    queryFn: () => getUnreadCount(profile?.id),
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
    activeNavItem,
  };
}

export type AppShellState = ReturnType<typeof useAppShell>;
