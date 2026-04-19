import type { ButtonHTMLAttributes } from 'react';
import { cn } from '../../lib/cn';

type ButtonVariant = 'default' | 'ghost' | 'icon' | 'link' | 'outline';
type ButtonSize = 'sm' | 'md' | 'icon';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  active?: boolean;
}

export function Button({
  variant = 'default',
  size = 'md',
  active,
  className,
  children,
  ...props
}: ButtonProps) {
  return (
    <button
      className={cn(
        'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-xl font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/70 disabled:pointer-events-none disabled:opacity-60',
        size === 'md' && 'h-10 px-4 text-sm',
        size === 'sm' && 'h-8 px-3 text-xs',
        size === 'icon' && 'h-9 w-9 shrink-0 rounded-lg p-0',
        variant === 'default' &&
          'bg-[image:var(--gradient-button)] text-white shadow-[0_18px_40px_rgba(0,0,0,0.24)] hover:opacity-95',
        variant === 'ghost' &&
          'border border-transparent bg-white/[0.03] text-foreground hover:bg-white/[0.06]',
        variant === 'icon' &&
          'border border-border bg-white/[0.03] text-muted-foreground hover:bg-white/[0.06] hover:text-foreground',
        variant === 'link' &&
          'h-auto rounded-none p-0 text-sm text-primary underline-offset-4 hover:underline',
        variant === 'outline' &&
          'border border-border bg-transparent text-muted-foreground hover:bg-white/[0.05] hover:text-foreground',
        variant === 'outline' &&
          active &&
          'border-primary bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground',
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
