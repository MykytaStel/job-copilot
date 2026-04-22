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
        'space-y-5 rounded-[var(--radius-card)] border border-border bg-card/85 p-7',
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
        'overflow-hidden rounded-[var(--radius-hero)] border border-border bg-card/85 shadow-[var(--shadow-hero)]',
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
      className={cn('rounded-2xl border border-border/70 bg-surface-muted p-4', className)}
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
      className={cn('rounded-2xl border border-border/70 bg-white-a04 px-4 py-3', className)}
      {...props}
    />
  );
}
