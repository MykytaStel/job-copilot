import type { ComponentPropsWithoutRef, ElementType, ReactNode } from 'react';
import { cn } from '../../lib/cn';

type SurfaceProps<E extends ElementType> = {
  as?: E;
  className?: string;
  children?: ReactNode;
} & Omit<ComponentPropsWithoutRef<E>, 'as' | 'className' | 'children'>;

export function SurfaceSection<E extends ElementType = 'section'>({
  as,
  className,
  ...props
}: SurfaceProps<E>) {
  const Component = (as ?? 'section') as ElementType;
  return (
    <Component
      className={cn(
        'space-y-5 rounded-[var(--radius-card)] border border-border/80 bg-card p-5 shadow-[var(--shadow-card)] sm:p-6',
        className,
      )}
      {...props}
    />
  );
}

export function SurfaceHero<E extends ElementType = 'div'>({
  as,
  className,
  ...props
}: SurfaceProps<E>) {
  const Component = (as ?? 'div') as ElementType;
  return (
    <Component
      className={cn(
        'overflow-hidden rounded-[var(--radius-hero)] border border-border/80 bg-card shadow-[var(--shadow-hero)]',
        className,
      )}
      {...props}
    />
  );
}

export function SurfaceInset<E extends ElementType = 'div'>({
  as,
  className,
  ...props
}: SurfaceProps<E>) {
  const Component = (as ?? 'div') as ElementType;
  return (
    <Component
      className={cn('rounded-[var(--radius-xl)] border border-border/60 bg-surface-muted p-4', className)}
      {...props}
    />
  );
}

export function SurfaceMetric<E extends ElementType = 'div'>({
  as,
  className,
  ...props
}: SurfaceProps<E>) {
  const Component = (as ?? 'div') as ElementType;
  return (
    <Component
      className={cn('rounded-[var(--radius-xl)] border border-border/60 bg-white-a04 px-4 py-3', className)}
      {...props}
    />
  );
}
