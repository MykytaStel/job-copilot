import type { HTMLAttributes } from 'react';
import { cn } from '../../lib/cn';

type BadgeVariant = 'default' | 'success' | 'danger' | 'warning' | 'muted';

const variantClass: Record<BadgeVariant, string> = {
  default: 'bg-surface-accent text-content-accent',
  success: 'bg-surface-success text-content-success',
  danger: 'bg-surface-danger text-content-danger',
  warning: 'bg-surface-warning text-content-warning',
  muted: 'bg-white/[.08] text-content-muted',
};

interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant;
}

export function Badge({ variant = 'default', className, children, ...props }: BadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-2 rounded-full px-3 py-1.5 text-sm',
        variantClass[variant],
        className,
      )}
      {...props}
    >
      {children}
    </span>
  );
}
