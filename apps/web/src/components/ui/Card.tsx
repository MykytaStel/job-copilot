import type { HTMLAttributes } from 'react';
import { cn } from '../../lib/cn';

export function Card({ className, children, ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        'bg-surface border border-edge-subtle rounded-[var(--radius-lg)]',
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
}

export function CardHeader({ className, children, ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn('flex items-start justify-between gap-4 px-3.5 pt-3.5 pb-3', className)}
      {...props}
    >
      {children}
    </div>
  );
}

export function CardContent({ className, children, ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div className={cn('px-3.5 pb-3.5', className)} {...props}>
      {children}
    </div>
  );
}
