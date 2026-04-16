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
 *   - Button size="icon" → local IconBtn (our Button has no icon variant yet)
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
 * Icon-only ghost button for the shell.
 * Defined locally because ui/Button.tsx does not yet have a size="icon" variant.
 * Inline background:transparent overrides the global `button { background: gradient }` base rule.
 */
function IconBtn({
  onClick,
  title,
  className,
  children,
}: {
  onClick?: () => void;
  title?: string;
  className?: string;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      title={title}
      onClick={onClick}
      style={{ background: 'transparent', border: 'none', padding: 0 }}
      className={cn(
        'inline-flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-lg',
        'text-content-muted hover:bg-surface-hover hover:text-content transition-colors',
        className,
      )}
    >
      {children}
    </button>
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
      className={({ isActive }) =>
        cn(
          'flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium no-underline transition-colors',
          isActive
            ? 'bg-surface-accent text-content-accent'
            : 'text-content-muted hover:bg-surface-hover hover:text-content',
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
    <div className="min-h-screen" style={{ background: 'var(--color-bg-app)' }}>

      {/* ── Desktop Sidebar ─────────────────────────────────────────────────── */}
      <aside
        className={cn(
          'fixed left-0 top-0 z-40 hidden h-screen flex-col border-r border-edge-subtle lg:flex transition-all duration-300',
          sidebarCollapsed ? 'w-16' : 'w-64',
        )}
        style={{ background: 'var(--color-bg-elevated)' }}
      >
        {/* Logo row */}
        <div
          className={cn(
            'flex h-16 flex-shrink-0 items-center border-b border-edge-subtle px-4',
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
              <span className="text-lg font-semibold text-content">Job Copilot</span>
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
            <IconBtn
              title="Collapse sidebar"
              onClick={() => setSidebarCollapsed(true)}
            >
              <ChevronLeft className="h-4 w-4" />
            </IconBtn>
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
          <div className="flex-shrink-0 border-t border-edge-subtle p-2">
            <button
              type="button"
              title="Expand sidebar"
              onClick={() => setSidebarCollapsed(false)}
              style={{ background: 'transparent', border: 'none' }}
              className="flex h-10 w-full items-center justify-center rounded-lg text-content-muted hover:bg-surface-hover hover:text-content transition-colors"
            >
              <Menu className="h-4 w-4" />
            </button>
          </div>
        )}

        {/* User section — placeholder, wire to profile store in a later slice */}
        {!sidebarCollapsed && (
          <div className="flex-shrink-0 border-t border-edge-subtle p-4">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-full bg-surface-accent text-content-accent">
                <span className="text-sm font-medium">JC</span>
              </div>
              <div className="min-w-0 flex-1">
                {/* TODO: replace with profile name/email from store */}
                <p className="truncate text-sm font-medium text-content">Job Copilot</p>
                <p className="truncate text-xs text-content-muted">operator dashboard</p>
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
          'fixed left-0 top-0 z-40 flex h-screen w-64 flex-col border-r border-edge-subtle transition-transform duration-300 lg:hidden',
          mobileNavOpen ? 'translate-x-0' : '-translate-x-full',
        )}
        style={{ background: 'var(--color-bg-elevated)' }}
      >
        <div className="flex h-14 flex-shrink-0 items-center border-b border-edge-subtle px-4">
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
            <span className="text-lg font-semibold text-content">Job Copilot</span>
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
        className="fixed left-0 right-0 top-0 z-20 flex h-14 flex-shrink-0 items-center justify-between border-b border-edge-subtle px-4 backdrop-blur-sm lg:hidden"
        style={{ background: 'rgba(13,19,32,0.95)' }}
      >
        <IconBtn title="Open menu" onClick={() => setMobileNavOpen(true)}>
          <Menu className="h-5 w-5" />
        </IconBtn>

        <Link to="/" className="flex items-center gap-2 no-underline">
          <div
            className="flex h-7 w-7 items-center justify-center rounded-lg"
            style={{ background: 'var(--gradient-button)' }}
          >
            <Sparkles className="h-3.5 w-3.5 text-white" />
          </div>
          <span className="font-semibold text-content">Job Copilot</span>
        </Link>

        {/* Placeholder — no notifications route */}
        <IconBtn title="Notifications (not yet implemented)">
          <Bell className="h-5 w-5" />
        </IconBtn>
      </header>

      {/* ── Desktop: Top Header ──────────────────────────────────────────────── */}
      <header
        className={cn(
          'fixed right-0 top-0 z-30 hidden h-16 items-center justify-between border-b border-edge-subtle px-6 backdrop-blur-sm lg:flex transition-all duration-300',
          sidebarCollapsed ? 'left-16' : 'left-64',
        )}
        style={{ background: 'rgba(13,19,32,0.95)' }}
      >
        {/* Global search — placeholder; no global search API exists yet */}
        <div className="relative max-w-md flex-1">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-content-muted pointer-events-none" />
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
          <IconBtn title="Notifications (not yet implemented)">
            <Bell className="h-5 w-5" />
          </IconBtn>
          {/* Placeholder — no settings route */}
          <IconBtn title="Settings (not yet implemented)">
            <Settings className="h-5 w-5" />
          </IconBtn>
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
