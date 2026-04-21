import { ChevronLeft } from 'lucide-react';

import { Button } from '../components/ui/Button';
import { cn } from '../lib/cn';
import { AppShellBrand } from './AppShellBrand';
import type { NavItem } from './navigation';
import { navigation } from './navigation';
import { SideNavItem } from './SideNavItem';

export function AppShellMobileNav({
  mobileNavOpen,
  setMobileNavOpen,
  activeNavItem,
}: {
  mobileNavOpen: boolean;
  setMobileNavOpen: (value: boolean) => void;
  activeNavItem: NavItem | null;
}) {
  return (
    <>
      {mobileNavOpen && (
        <div
          className="fixed inset-0 z-30 bg-black/50 lg:hidden"
          onClick={() => setMobileNavOpen(false)}
        />
      )}

      <div
        className={cn(
          'fixed left-0 top-0 z-40 flex h-screen w-64 flex-col border-r border-sidebar-border bg-sidebar transition-transform duration-300 lg:hidden',
          mobileNavOpen ? 'translate-x-0' : '-translate-x-full',
        )}
      >
        <div className="flex h-14 flex-shrink-0 items-center justify-between border-b border-sidebar-border px-4">
          <AppShellBrand onClick={() => setMobileNavOpen(false)} />
          <Button
            variant="ghost"
            size="icon"
            className="h-9 w-9 text-sidebar-foreground/70 hover:bg-sidebar-accent hover:text-sidebar-foreground"
            onClick={() => setMobileNavOpen(false)}
            title="Close menu"
          >
            <ChevronLeft className="h-4 w-4" />
          </Button>
        </div>
        <nav className="flex-1 space-y-0.5 overflow-y-auto p-2">
          {navigation.map((item) => (
            <SideNavItem key={item.to} item={item} onClick={() => setMobileNavOpen(false)} />
          ))}
        </nav>
        <div className="border-t border-sidebar-border p-4">
          <div className="rounded-2xl border border-white/5 bg-sidebar-accent/60 px-4 py-3">
            <p className="m-0 text-[11px] font-semibold uppercase tracking-[0.14em] text-sidebar-foreground/45">
              Current view
            </p>
            <p className="m-0 mt-2 text-sm font-medium text-sidebar-foreground">
              {activeNavItem?.name ?? 'Details'}
            </p>
          </div>
        </div>
      </div>
    </>
  );
}
