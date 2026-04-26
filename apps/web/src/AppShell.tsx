import { Suspense } from 'react';
import { Outlet } from 'react-router-dom';
import { cn } from './lib/cn';
import { AppShellDesktopSidebar } from './app-shell/AppShellDesktopSidebar';
import { AppShellMobileNav } from './app-shell/AppShellMobileNav';
import { AppShellTopHeader } from './app-shell/AppShellTopHeader';
import { MlDegradedBanner } from './app-shell/MlDegradedBanner';
import { RouteSkeleton } from './app-shell/RouteSkeleton';
import { useAppShell } from './app-shell/useAppShell';

export default function AppShell() {
  const state = useAppShell();

  return (
    <div className="min-h-screen bg-background">
      <AppShellDesktopSidebar
        sidebarCollapsed={state.sidebarCollapsed}
        setSidebarCollapsed={state.setSidebarCollapsed}
        profileLoading={state.profileLoading}
        profileName={state.profile?.name}
        profileEmail={state.profile?.email}
      />

      <AppShellMobileNav
        mobileNavOpen={state.mobileNavOpen}
        setMobileNavOpen={state.setMobileNavOpen}
        activeNavItem={state.activeNavItem}
      />

      <AppShellTopHeader
        location={state.location}
        activeNavItem={state.activeNavItem}
        sidebarCollapsed={state.sidebarCollapsed}
        setMobileNavOpen={state.setMobileNavOpen}
        unreadCount={state.unreadCount}
      />

      <main
        className={cn(
          'min-h-screen pt-14 lg:pt-16 transition-all duration-300',
          state.sidebarCollapsed ? 'lg:pl-16' : 'lg:pl-64',
        )}
      >
        <div className="p-4 lg:p-8 xl:p-10">
          {state.mlDegraded && (
            <MlDegradedBanner onDismiss={state.dismissMlBanner} />
          )}
          <Suspense fallback={<RouteSkeleton />}>
            <Outlet />
          </Suspense>
        </div>
      </main>
    </div>
  );
}
