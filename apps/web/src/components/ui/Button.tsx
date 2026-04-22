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
        'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-[var(--radius-lg)] border text-sm font-semibold tracking-[0.01em] transition-[background-color,border-color,color,box-shadow,opacity,transform] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/45 focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60',
        size === 'md' && 'h-10 px-4',
        size === 'sm' && 'h-9 px-3.5 text-xs',
        size === 'icon' && 'h-9 w-9 shrink-0 rounded-[var(--radius-md)] p-0',
        variant === 'default' &&
          'border-transparent bg-[image:var(--gradient-button)] text-primary-foreground shadow-[0_18px_40px_rgba(0,0,0,0.24)] hover:brightness-105 active:translate-y-px',
        variant === 'ghost' &&
          'border-transparent bg-white/[0.03] text-foreground hover:bg-white/[0.07] hover:text-foreground',
        variant === 'icon' &&
          'border-border/70 bg-white/[0.03] text-muted-foreground hover:bg-white/[0.07] hover:text-foreground',
        variant === 'link' &&
          'h-auto rounded-none border-transparent bg-transparent p-0 text-sm font-medium text-primary shadow-none underline-offset-4 hover:text-primary hover:underline',
        variant === 'outline' &&
          'border-border bg-transparent text-foreground hover:bg-white/[0.05] hover:text-foreground',
        variant === 'outline' &&
          active &&
          'border-primary/40 bg-primary/12 text-primary hover:bg-primary/18 hover:text-primary',
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
