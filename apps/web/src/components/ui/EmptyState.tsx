import type { ReactNode } from 'react';
import { cn } from '../../lib/cn';

export function EmptyState({
  message,
  description,
  icon,
  className,
}: {
  message: string;
  description?: string;
  icon?: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        'rounded-2xl border border-border bg-card/70 px-5 py-7 text-center text-sm text-muted-foreground',
        className,
      )}
    >
      {icon && <div className="mb-3 flex justify-center text-primary">{icon}</div>}
      <p className="m-0 font-medium text-card-foreground">{message}</p>
      {description && (
        <p className="m-0 mt-2 text-xs leading-6 text-muted-foreground">{description}</p>
      )}
    </div>
  );
}
