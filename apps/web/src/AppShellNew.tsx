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
 *   - Bell icon button: no notifications route or API
 *   - Settings icon button: no settings route
 *   - Header search input: no global search API (readOnly + tabIndex=-1)
 *   - User section footer: hardcoded; should wire to profile store
 */

import { useState } from 'react';
import { Link, NavLink, Outlet } from 'react-router-dom';
import { Toaster } from 'react-hot-toast';
import type { LucideIcon } from 'lucide-react';
import {
  BarChart3,
  Bell,
  ChevronLeft,
  KanbanSquare,
  LayoutDashboard,
  Menu,
  MessageSquare,
  Search,
  Settings,
  Sparkles,
  User,
} from 'lucide-react';
import { cn } from './lib/cn';
import { Button } from './components/ui/Button';

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
  { name: 'Profile',      to: '/profile',      end: false, icon: User            },
];

// ── Local primitives ──────────────────────────────────────────────────────────


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
      className={({ isActive }) =>
        cn(
          'flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium no-underline transition-colors',
          isActive
            ? 'bg-sidebar-accent text-sidebar-primary'
            : 'text-sidebar-foreground/70 hover:bg-sidebar-accent hover:text-sidebar-foreground',
          collapsed && 'justify-center px-2',
        )
      }
    >
      <item.icon className="h-5 w-5 shrink-0" />
      {!collapsed && <span>{item.name}</span>}
    </NavLink>
  );
}

// ── Shell ─────────────────────────────────────────────────────────────────────

export default function AppShellNew() {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [mobileNavOpen, setMobileNavOpen] = useState(false);

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
              <ChevronLeft className="h-4 w-4" />
            </Button>
          )}
        </div>

        {/* Nav links */}
        <nav className="flex-1 space-y-0.5 overflow-y-auto p-2">
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
              <Menu className="h-4 w-4" />
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
                {/* TODO: replace with profile name/email from store */}
                <p className="truncate text-sm font-medium text-sidebar-foreground">Job Copilot</p>
                <p className="truncate text-xs text-sidebar-foreground/60">operator dashboard</p>
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
        <div className="flex h-14 flex-shrink-0 items-center border-b border-sidebar-border px-4">
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

        <Link to="/" className="flex items-center gap-2 no-underline">
          <div
            className="flex h-7 w-7 items-center justify-center rounded-lg"
            style={{ background: 'var(--gradient-button)' }}
          >
            <Sparkles className="h-3.5 w-3.5 text-white" />
          </div>
          <span className="font-semibold text-sidebar-foreground">Job Copilot</span>
        </Link>

        {/* Placeholder — no notifications route */}
        <Button variant="ghost" size="icon" title="Notifications (not yet implemented)" className="h-9 w-9">
          <Bell className="h-5 w-5" />
        </Button>
      </header>

      {/* ── Desktop: Top Header ──────────────────────────────────────────────── */}
      <header
        className={cn(
          'fixed right-0 top-0 z-30 hidden h-16 items-center justify-between border-b border-border bg-background/95 px-6 backdrop-blur-sm lg:flex transition-all duration-300',
          sidebarCollapsed ? 'left-16' : 'left-64',
        )}
      >
        {/* Global search — placeholder; no global search API exists yet */}
        <div className="relative max-w-md flex-1">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground pointer-events-none" />
          <input
            type="search"
            placeholder="Search jobs, companies, skills…"
            readOnly
            tabIndex={-1}
            title="Global search (not yet implemented)"
            style={{ paddingLeft: 36 }}
          />
        </div>

        <div className="flex items-center gap-1">
          {/* Placeholder — no notifications route */}
          <Button
            variant="ghost"
            size="icon"
            title="Notifications (not yet implemented)"
            className="h-9 w-9 text-sidebar-foreground hover:bg-sidebar-accent"
          >
            <Bell className="h-5 w-5" />
          </Button>
          {/* Placeholder — no settings route */}
          <Button
            variant="ghost"
            size="icon"
            title="Settings (not yet implemented)"
            className="h-9 w-9 text-sidebar-foreground hover:bg-sidebar-accent"
          >
            <Settings className="h-5 w-5" />
          </Button>
        </div>
      </header>

      {/* ── Main content area ────────────────────────────────────────────────── */}
      <main
        className={cn(
          'min-h-screen pt-14 lg:pt-16 transition-all duration-300',
          sidebarCollapsed ? 'lg:pl-16' : 'lg:pl-64',
        )}
      >
        <div className="p-4 lg:p-10">
          <Outlet />
        </div>
      </main>

      <Toaster
        position="bottom-right"
        toastOptions={{ duration: 3000, style: { fontSize: 14 } }}
      />
    </div>
  );
}
