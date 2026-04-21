import { Bell } from 'lucide-react';
import { Link } from 'react-router-dom';

import { cn } from '../lib/cn';

export function NotificationIconButton({
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
      title={unreadCount > 0 ? `Notifications (${unreadCount} unread)` : 'Notifications'}
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
