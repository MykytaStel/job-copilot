import type { HTMLAttributes } from 'react';
import { cn } from '../../lib/cn';
import { semanticBadgeClass } from './semanticTone';

type BadgeVariant = 'default' | 'info' | 'success' | 'danger' | 'warning' | 'muted';

const variantClass: Record<BadgeVariant, string> = {
  default: semanticBadgeClass.primary,
  info: semanticBadgeClass.info,
  success: semanticBadgeClass.success,
  danger: semanticBadgeClass.danger,
  warning: semanticBadgeClass.warning,
  muted: semanticBadgeClass.muted,
};

interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant;
}

export function Badge({ variant = 'default', className, children, ...props }: BadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-sm',
        variantClass[variant],
        className,
      )}
      {...props}
    >
      {children}
    </span>
  );
}
