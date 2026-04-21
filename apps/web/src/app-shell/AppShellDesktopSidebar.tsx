import { PanelLeftClose, PanelLeftOpen } from 'lucide-react';

import { Button } from '../components/ui/Button';
import { cn } from '../lib/cn';
import { AppShellBrand, AppShellBrandMark } from './AppShellBrand';
import { AppShellProfileBadge } from './AppShellProfileBadge';
import { navigation } from './navigation';
import { SideNavItem } from './SideNavItem';

export function AppShellDesktopSidebar({
  sidebarCollapsed,
  setSidebarCollapsed,
  profileLoading,
  profileName,
  profileEmail,
}: {
  sidebarCollapsed: boolean;
  setSidebarCollapsed: (value: boolean) => void;
  profileLoading: boolean;
  profileName?: string | null;
  profileEmail?: string | null;
}) {
  return (
    <aside
      className={cn(
        'fixed left-0 top-0 z-40 hidden h-screen flex-col border-r border-sidebar-border bg-sidebar lg:flex transition-all duration-300',
        sidebarCollapsed ? 'w-16' : 'w-64',
      )}
    >
      <div
        className={cn(
          'flex h-16 flex-shrink-0 items-center border-b border-sidebar-border px-4',
          sidebarCollapsed ? 'justify-center' : 'justify-between',
        )}
      >
        {!sidebarCollapsed ? <AppShellBrand /> : <AppShellBrandMark />}
        {!sidebarCollapsed && (
          <Button
            variant="ghost"
            size="icon"
            title="Collapse sidebar"
            className="h-8 w-8 text-sidebar-foreground hover:bg-sidebar-accent"
            onClick={() => setSidebarCollapsed(true)}
          >
            <PanelLeftClose className="h-4 w-4" />
          </Button>
        )}
      </div>

      <div className="px-4 pt-4">
        {!sidebarCollapsed && (
          <p className="m-0 text-[10px] font-semibold uppercase tracking-[0.16em] text-sidebar-foreground/40">
            Workspace
          </p>
        )}
      </div>
      <nav className="flex-1 space-y-1 overflow-y-auto p-2">
        {navigation.map((item) => (
          <SideNavItem key={item.to} item={item} collapsed={sidebarCollapsed} />
        ))}
      </nav>

      {sidebarCollapsed && (
        <div className="flex-shrink-0 border-t border-sidebar-border p-2">
          <Button
            type="button"
            variant="ghost"
            size="icon"
            title="Expand sidebar"
            onClick={() => setSidebarCollapsed(false)}
            className="h-10 w-full text-sidebar-foreground/60 hover:bg-sidebar-accent hover:text-sidebar-foreground"
          >
            <PanelLeftOpen className="h-4 w-4" />
          </Button>
        </div>
      )}

      {!sidebarCollapsed && (
        <div className="flex-shrink-0 border-t border-sidebar-border p-4">
          <AppShellProfileBadge
            loading={profileLoading}
            name={profileName}
            email={profileEmail}
          />
        </div>
      )}
    </aside>
  );
}
