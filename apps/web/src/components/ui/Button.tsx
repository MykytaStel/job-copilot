import type { ButtonHTMLAttributes } from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '../../lib/cn';

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-[var(--radius-lg)] border text-sm font-semibold tracking-[0.01em] transition-[background-color,border-color,color,box-shadow,opacity,transform] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/45 focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60',
  {
    variants: {
      variant: {
        default:
          'border-transparent bg-[image:var(--gradient-button)] text-primary-foreground shadow-[0_18px_40px_rgba(0,0,0,0.24)] hover:brightness-105 active:translate-y-px',
        ghost:
          'border-transparent bg-surface-muted text-foreground hover:bg-white-a07 hover:text-foreground',
        icon: 'border-border/70 bg-surface-muted text-muted-foreground hover:bg-white-a07 hover:text-foreground',
        link: 'h-auto rounded-none border-transparent bg-transparent p-0 text-sm font-medium text-primary shadow-none underline-offset-4 hover:text-primary hover:underline',
        outline:
          'border-border bg-transparent text-foreground hover:bg-white-a05 hover:text-foreground',
      },
      size: {
        md: 'h-10 px-4',
        sm: 'h-9 px-3.5 text-xs',
        icon: 'h-9 w-9 shrink-0 rounded-[var(--radius-md)] p-0',
      },
      active: {
        true: '',
        false: '',
      },
    },
    compoundVariants: [
      {
        variant: 'outline',
        active: true,
        className:
          'border-primary/40 bg-primary/12 text-primary hover:bg-primary/18 hover:text-primary',
      },
    ],
    defaultVariants: { variant: 'default', size: 'md', active: false },
  },
);

interface ButtonProps
  extends ButtonHTMLAttributes<HTMLButtonElement>,
    Omit<VariantProps<typeof buttonVariants>, 'active'> {
  active?: boolean;
}

export function Button({
  variant,
  size,
  active = false,
  className,
  children,
  ...props
}: ButtonProps) {
  return (
    <button className={cn(buttonVariants({ variant, size, active }), className)} {...props}>
      {children}
    </button>
  );
}
