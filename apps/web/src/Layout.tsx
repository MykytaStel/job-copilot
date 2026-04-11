import { NavLink, Outlet } from 'react-router-dom';
import { Toaster } from 'react-hot-toast';
import type { LucideIcon } from 'lucide-react';
import { KanbanSquare, LayoutDashboard, User } from 'lucide-react';

type NavLinkItem = {
  to: string;
  label: string;
  icon: LucideIcon;
};

const links: NavLinkItem[] = [
  { to: '/', label: 'Dashboard', icon: LayoutDashboard },
  { to: '/applications', label: 'Applications', icon: KanbanSquare },
  { to: '/profile', label: 'Profile', icon: User },
];

export default function Layout() {
  return (
    <div className="appShell">
      <nav className="sidebar">
        <p className="navBrand">Job Copilot UA</p>
        <p className="muted" style={{ fontSize: 12, marginBottom: 16 }}>
          `engine-api` mode
        </p>
        <ul className="navList">
          {links.map((link) => (
            <li key={link.to}>
              <NavLink
                to={link.to}
                end={link.to === '/'}
                className={({ isActive }) =>
                  isActive ? 'navLink active' : 'navLink'
                }
                style={{ display: 'flex', alignItems: 'center', gap: 8 }}
              >
                <link.icon size={16} style={{ flexShrink: 0 }} />
                {link.label}
              </NavLink>
            </li>
          ))}
        </ul>
      </nav>
      <main className="content">
        <Outlet />
      </main>
      <Toaster
        position="bottom-right"
        toastOptions={{
          duration: 3000,
          style: { fontSize: 14 },
        }}
      />
    </div>
  );
}
