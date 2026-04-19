/**
 * AppShellNew — layout migration candidate (Phase 1 preview).
 *
 * This file is ADDITIVE ONLY. The existing Layout.tsx is untouched.
 * To preview: temporarily swap `Layout` for `AppShellNew` in App.tsx,
 * then revert. Permanent swap happens in a dedicated later step.
 *
 * Adapted from: design-reference/job_copilot_design_model/components/app-shell.tsx
 * Changes from reference:
 *   - Removed "use client" (not applicable in React)
 *   - next/link → Link / NavLink from react-router-dom
 *   - usePathname → NavLink active prop (no useLocation needed)
 *   - children prop → <Outlet />
 *   - Sheet (shadcn) → state-driven fixed drawer (no Sheet dependency)
 *   - Button size="icon" is supported by our local Button
 *   - Input (shadcn) → raw <input> with existing base styles
 *   - @/ alias → relative imports
 *   - Navigation targets mapped to real routes (see ROUTE MAPPING below)
 *
 * ROUTE MAPPING:
 *   Design "/":          → "/"              Dashboard         ✓ exists
 *   Design "/profile":   → "/profile"       Profile           ✓ exists
 *   Design "/job/1":     → NOT a nav item   JobDetails is a detail route (/jobs/:id)
 *   Design "/feedback":  → "/feedback"      FeedbackCenter    ✓ exists
 *   Design "/analytics": → "/analytics"     Analytics         ✓ exists
 *   (added) n/a          → "/applications"  ApplicationBoard  ✓ exists, was missing from design
 *
 * PLACEHOLDERS (not yet wired):
 *   - Settings icon button: no settings route
 *   - User section footer: hardcoded; should wire to profile store
 */

import { Suspense, useMemo, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Link, NavLink, Outlet, useLocation } from 'react-router-dom';
import { Toaster } from 'react-hot-toast';
import type { LucideIcon } from 'lucide-react';
import {
  BarChart3,
  Bell,
  ChevronLeft,
  ChevronRight,
  KanbanSquare,
  LayoutDashboard,
  LineChart,
  Menu,
  MessageSquare,
  PanelLeftClose,
  PanelLeftOpen,
  Settings,
  Sparkles,
  User,
} from 'lucide-react';
import { cn } from './lib/cn';
import { Button } from './components/ui/Button';
import { GlobalSearch } from './components/GlobalSearch';
import { getProfile, getUnreadCount } from './api';
import { queryKeys } from './queryKeys';

// ── Navigation config ──────────────────────────────────────────────────────────

type NavItem = {
  name: string;
  to: string;
  end: boolean;
  icon: LucideIcon;
};

const navigation: NavItem[] = [
  { name: 'Dashboard',    to: '/',             end: true,  icon: LayoutDashboard },
  { name: 'Applications', to: '/applications', end: false, icon: KanbanSquare    },
  { name: 'Feedback',     to: '/feedback',     end: false, icon: MessageSquare   },
  { name: 'Analytics',    to: '/analytics',    end: false, icon: BarChart3       },
  { name: 'Market',       to: '/market',       end: false, icon: LineChart       },
  { name: 'Profile',      to: '/profile',      end: false, icon: User            },
];

// ── Local primitives ──────────────────────────────────────────────────────────

function RouteSkeleton() {
  return (
    <div className="space-y-8">
      <div className="space-y-3">
        <div className="h-4 w-40 rounded-full bg-white/[0.05]" />
        <div className="h-10 w-96 max-w-full rounded-2xl bg-white/[0.06]" />
        <div className="h-4 w-[42rem] max-w-full rounded-full bg-white/[0.04]" />
      </div>
      <div className="grid gap-4 md:grid-cols-3">
        {Array.from({ length: 3 }).map((_, index) => (
          <div
            key={index}
            className="h-32 rounded-[24px] border border-border/70 bg-card/70 animate-pulse"
          />
        ))}
      </div>
      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
        <div className="space-y-6">
          <div className="h-72 rounded-[24px] border border-border/70 bg-card/70 animate-pulse" />
          <div className="h-56 rounded-[24px] border border-border/70 bg-card/70 animate-pulse" />
        </div>
        <div className="h-80 rounded-[24px] border border-border/70 bg-card/70 animate-pulse" />
      </div>
    </div>
  );
}

function NotificationIconButton({
  unreadCount,
  mobile = false,
}: {
  unreadCount: number;
  mobile?: boolean;
}) {
  const badgeLabel = unreadCount > 99 ? '99+' : String(unreadCount);
  const linkClass = mobile
    ? 'border border-transparent bg-white/[0.03] text-foreground hover:bg-white/[0.06]'
    : 'border border-border bg-white/[0.03] text-muted-foreground hover:bg-white/[0.06] hover:text-foreground';

  return (
    <Link
      to="/notifications"
      title={
        unreadCount > 0
          ? `Notifications (${unreadCount} unread)`
          : 'Notifications'
      }
      className={cn(
        'relative inline-flex h-9 w-9 items-center justify-center rounded-lg no-underline transition-colors',
        linkClass,
      )}
    >
      <Bell className="h-5 w-5" />
      {unreadCount > 0 && (
        <span className="pointer-events-none absolute -right-1 -top-1 inline-flex min-w-[18px] items-center justify-center rounded-full border border-background bg-primary px-1.5 py-0.5 text-[10px] font-semibold leading-none text-primary-foreground shadow-sm">
          {badgeLabel}
        </span>
      )}
    </Link>
  );
}

/**
 * A single sidebar nav link. Uses NavLink's render-prop API for active state
 * so no useLocation / usePathname is needed.
 */
function SideNavItem({
  item,
  collapsed,
  onClick,
}: {
  item: NavItem;
  collapsed?: boolean;
  onClick?: () => void;
}) {
  return (
    <NavLink
      to={item.to}
      end={item.end}
      onClick={onClick}
      title={collapsed ? item.name : undefined}
      className={({ isActive }) =>
        cn(
          'group relative flex items-center gap-3 rounded-xl border border-transparent px-3 py-2.5 text-sm font-medium no-underline transition-colors',
          isActive
            ? 'border-sidebar-primary/20 bg-sidebar-accent text-sidebar-primary shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]'
            : 'text-sidebar-foreground/70 hover:border-white/5 hover:bg-sidebar-accent hover:text-sidebar-foreground',
          collapsed && 'justify-center px-2',
        )
      }
    >
      {({ isActive }) => (
        <>
          {isActive && !collapsed && (
            <span className="absolute left-1 top-1/2 h-6 w-1 -translate-y-1/2 rounded-full bg-sidebar-primary/70" />
          )}
      <item.icon className="h-5 w-5 shrink-0 transition-colors group-hover:text-sidebar-foreground" />
          {!collapsed && <span>{item.name}</span>}
        </>
      )}
    </NavLink>
  );
}

// ── Shell ─────────────────────────────────────────────────────────────────────

export default function AppShellNew() {
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

  return (
    <div className="min-h-screen bg-background">

      {/* ── Desktop Sidebar ─────────────────────────────────────────────────── */}
      <aside
        className={cn(
          'fixed left-0 top-0 z-40 hidden h-screen flex-col border-r border-sidebar-border bg-sidebar lg:flex transition-all duration-300',
          sidebarCollapsed ? 'w-16' : 'w-64',
        )}
      >
        {/* Logo row */}
        <div
          className={cn(
            'flex h-16 flex-shrink-0 items-center border-b border-sidebar-border px-4',
            sidebarCollapsed ? 'justify-center' : 'justify-between',
          )}
        >
          {!sidebarCollapsed && (
            <Link to="/" className="flex items-center gap-2 no-underline">
              <div
                className="flex h-8 w-8 items-center justify-center rounded-lg"
                style={{ background: 'var(--gradient-button)' }}
              >
                <Sparkles className="h-4 w-4 text-white" />
              </div>
              <span className="text-lg font-semibold text-sidebar-foreground">Job Copilot</span>
            </Link>
          )}
          {sidebarCollapsed && (
            <div
              className="flex h-8 w-8 items-center justify-center rounded-lg"
              style={{ background: 'var(--gradient-button)' }}
            >
              <Sparkles className="h-4 w-4 text-white" />
            </div>
          )}
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

        {/* Nav links */}
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

        {/* Expand button (visible only when collapsed) */}
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

        {/* User section — placeholder, wire to profile store in a later slice */}
        {!sidebarCollapsed && (
          <div className="flex-shrink-0 border-t border-sidebar-border p-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-full bg-sidebar-accent text-sidebar-primary">
                <span className="text-sm font-medium">JC</span>
              </div>
              <div className="min-w-0 flex-1">
                {profileLoading ? (
                  <>
                    <div className="mb-1 h-3.5 w-24 animate-pulse rounded bg-sidebar-accent" />
                    <div className="h-3 w-32 animate-pulse rounded bg-sidebar-accent" />
                  </>
                ) : (
                  <>
                    <p className="truncate text-sm font-medium text-sidebar-foreground">
                      {profile?.name ?? '—'}
                    </p>
                    <p className="truncate text-xs text-sidebar-foreground/60">
                      {profile?.email ?? '—'}
                    </p>
                  </>
                )}
              </div>
            </div>
          </div>
        )}
      </aside>

      {/* ── Mobile: Backdrop ─────────────────────────────────────────────────── */}
      {mobileNavOpen && (
        <div
          className="fixed inset-0 z-30 bg-black/50 lg:hidden"
          onClick={() => setMobileNavOpen(false)}
        />
      )}

      {/* ── Mobile: Side Drawer ──────────────────────────────────────────────── */}
      <div
        className={cn(
          'fixed left-0 top-0 z-40 flex h-screen w-64 flex-col border-r border-sidebar-border bg-sidebar transition-transform duration-300 lg:hidden',
          mobileNavOpen ? 'translate-x-0' : '-translate-x-full',
        )}
      >
        <div className="flex h-14 flex-shrink-0 items-center justify-between border-b border-sidebar-border px-4">
          <Link
            to="/"
            className="flex items-center gap-2 no-underline"
            onClick={() => setMobileNavOpen(false)}
          >
            <div
              className="flex h-8 w-8 items-center justify-center rounded-lg"
              style={{ background: 'var(--gradient-button)' }}
            >
              <Sparkles className="h-4 w-4 text-white" />
            </div>
            <span className="text-lg font-semibold text-sidebar-foreground">Job Copilot</span>
          </Link>
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
            <SideNavItem
              key={item.to}
              item={item}
              onClick={() => setMobileNavOpen(false)}
            />
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

      {/* ── Mobile: Top Header ───────────────────────────────────────────────── */}
      <header
        className="fixed left-0 right-0 top-0 z-20 flex h-14 flex-shrink-0 items-center justify-between border-b border-border bg-background/95 px-4 backdrop-blur-sm lg:hidden"
      >
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

      {/* ── Desktop: Top Header ──────────────────────────────────────────────── */}
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

          <GlobalSearch />
        </div>

        <div className="flex items-center gap-2">
          <NotificationIconButton unreadCount={unreadCount} />
          {/* Placeholder — no settings route */}
          <Button
            variant="icon"
            size="icon"
            title="Settings (not yet implemented)"
            className="h-9 w-9"
          >
            <Settings className="h-5 w-5" />
          </Button>
          <div className="ml-1 flex items-center gap-3 rounded-full border border-border bg-white/[0.03] px-3 py-1.5">
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

      {/* ── Main content area ────────────────────────────────────────────────── */}
      <main
        className={cn(
          'min-h-screen pt-14 lg:pt-16 transition-all duration-300',
          sidebarCollapsed ? 'lg:pl-16' : 'lg:pl-64',
        )}
      >
        <div className="p-4 lg:p-8 xl:p-10">
          <Suspense fallback={<RouteSkeleton />}>
            <Outlet />
          </Suspense>
        </div>
      </main>

      <Toaster
        position="bottom-right"
        toastOptions={{ duration: 3000, style: { fontSize: 14 } }}
      />
    </div>
  );
}
