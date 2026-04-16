import type { ButtonHTMLAttributes } from 'react';
import { cn } from '../../lib/cn';

type ButtonVariant = 'default' | 'ghost' | 'link';
type ButtonSize = 'sm' | 'md';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
}

export function Button({
  variant = 'default',
  size = 'md',
  className,
  children,
  ...props
}: ButtonProps) {
  return (
    <button
      className={cn(
        variant === 'default' && 'btn',
        variant === 'ghost' && 'ghostBtn',
        variant === 'ghost' && size === 'sm' && 'ghostBtnCompact',
        variant === 'link' && 'linkBtn',
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
