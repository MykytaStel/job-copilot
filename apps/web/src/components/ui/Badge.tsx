import type { HTMLAttributes } from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '../../lib/cn';

const badgeVariants = cva('inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-sm', {
  variants: {
    variant: {
      default: 'border-primary/25 bg-primary/12 text-primary',
      info: 'border-fit-good/25 bg-fit-good/12 text-fit-good',
      success: 'border-fit-excellent/25 bg-fit-excellent/12 text-fit-excellent',
      warning: 'border-fit-fair/25 bg-fit-fair/12 text-fit-fair',
      danger: 'border-destructive/25 bg-destructive/12 text-destructive',
      muted: 'border-border bg-white-a04 text-muted-foreground',
    },
  },
  defaultVariants: { variant: 'default' },
});

interface BadgeProps extends HTMLAttributes<HTMLSpanElement>, VariantProps<typeof badgeVariants> {}

export function Badge({ variant, className, children, ...props }: BadgeProps) {
  return (
    <span className={cn(badgeVariants({ variant }), className)} {...props}>
      {children}
    </span>
  );
}
