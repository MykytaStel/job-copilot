import type { HTMLAttributes, ReactNode } from 'react';
import { cn } from '../../lib/cn';

export function Page({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        'mx-auto flex w-full max-w-[1380px] flex-col gap-8 px-2 sm:px-3 lg:gap-10',
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
    return <div className={cn('grid gap-8', className)}>{children}</div>;
  }

  return (
    <div className={cn('grid gap-8 xl:grid-cols-[minmax(0,1fr)_360px]', className)}>
      <div className="min-w-0 space-y-8">{children}</div>
      <aside className="space-y-6">{aside}</aside>
    </div>
  );
}
