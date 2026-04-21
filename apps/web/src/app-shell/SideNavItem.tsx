import { NavLink } from 'react-router-dom';

import { cn } from '../lib/cn';
import type { NavItem } from './navigation';

export function SideNavItem({
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
