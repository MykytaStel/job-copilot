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
        variant === 'default' && 'btn',
        variant === 'ghost' && 'ghostBtn',
        variant === 'icon' && 'ghostBtn',
        variant === 'ghost' && size === 'sm' && 'ghostBtnCompact',
        variant === 'icon' && size === 'sm' && 'ghostBtnCompact',
        variant === 'link' && 'linkBtn',
        variant === 'outline' && 'btnOutline',
        variant === 'outline' && active && 'btnOutline-active',
        size === 'icon' && 'inline-flex h-8 w-8 shrink-0 items-center justify-center gap-0 rounded-lg p-0',
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
