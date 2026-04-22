import { Menu, Settings } from 'lucide-react';
import type { Location } from 'react-router-dom';
import { useNavigate } from 'react-router-dom';

import { Button } from '../components/ui/Button';
import { GlobalSearch } from '../components/GlobalSearch';
import { cn } from '../lib/cn';
import { NotificationIconButton } from './NotificationIconButton';
import type { NavItem } from './navigation';

export function AppShellTopHeader({
  location,
  activeNavItem,
  sidebarCollapsed,
  setMobileNavOpen,
  unreadCount,
}: {
  location: Location;
  activeNavItem: NavItem | null;
  sidebarCollapsed: boolean;
  setMobileNavOpen: (value: boolean) => void;
  unreadCount: number;
}) {
  const navigate = useNavigate();

  return (
    <>
      <header className="fixed left-0 right-0 top-0 z-20 flex h-14 flex-shrink-0 items-center justify-between border-b border-border bg-background/95 px-4 backdrop-blur-sm lg:hidden">
        <Button
          variant="ghost"
          size="icon"
          title="Open menu"
          className="h-9 w-9"
          onClick={() => setMobileNavOpen(true)}
        >
          <Menu className="h-5 w-5" />
        </Button>

        <div className="text-center">
          <p className="m-0 text-[10px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
            Job Copilot
          </p>
          <p className="m-0 text-sm font-semibold text-sidebar-foreground">
            {activeNavItem?.name ?? 'Detail View'}
          </p>
        </div>

        <NotificationIconButton unreadCount={unreadCount} mobile />
      </header>

      <header
        className={cn(
          'fixed right-0 top-0 z-30 hidden h-16 items-center justify-between border-b border-border bg-background/95 px-6 backdrop-blur-sm lg:flex transition-all duration-300',
          sidebarCollapsed ? 'left-16' : 'left-64',
        )}
      >
        <div className="flex min-w-0 flex-1 items-center gap-4">
          <div className="min-w-0">
            <p className="m-0 text-[10px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
              Current view
            </p>
            <div className="mt-1 flex items-center gap-2">
              <p className="m-0 truncate text-sm font-semibold text-foreground">
                {activeNavItem?.name ?? 'Detail View'}
              </p>
              {activeNavItem && (
                <span className="inline-flex items-center rounded-full border border-primary/20 bg-primary/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.14em] text-primary">
                  Active
                </span>
              )}
            </div>
          </div>

          <GlobalSearch key={location.pathname} />
        </div>

        <div className="flex items-center gap-2">
          <NotificationIconButton unreadCount={unreadCount} />
          <Button
            variant="icon"
            size="icon"
            title="Settings"
            className="h-9 w-9"
            onClick={() => navigate('/settings')}
          >
            <Settings className="h-5 w-5" />
          </Button>
          <div className="ml-1 flex items-center gap-3 rounded-full border border-border bg-surface-muted px-3 py-1.5">
            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/15 text-primary">
              <span className="text-xs font-semibold">JC</span>
            </div>
            <div className="min-w-0">
              <p className="m-0 text-xs font-medium text-foreground">Job Copilot</p>
              <p className="m-0 text-[11px] text-muted-foreground">operator dashboard</p>
            </div>
          </div>
        </div>
      </header>
    </>
  );
}
