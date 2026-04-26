import type { HTMLAttributes, ReactNode } from 'react';

import { cn } from '../../lib/cn';

export function Page({
  className,
  ...props
}: HTMLAttributes<HTMLElement>) {
  return (
    <main
      className={cn(
        'mx-auto w-full max-w-[1400px] min-w-0 overflow-x-hidden px-4 py-4 sm:px-6 sm:py-6 lg:px-8 lg:py-8',
        className,
      )}
      {...props}
    />
  );
}

export function PageGrid({
  className,
  children,
  aside,
}: {
  className?: string;
  children: ReactNode;
  aside?: ReactNode;
}) {
  if (!aside) {
    return <div className={cn('min-w-0', className)}>{children}</div>;
  }

  return (
    <div
      className={cn(
        'grid min-w-0 gap-6 lg:grid-cols-[minmax(0,1fr)_320px]',
        className,
      )}
    >
      <div className="min-w-0">{children}</div>
      <div className="min-w-0 lg:min-w-[280px]">{aside}</div>
    </div>
  );
}
