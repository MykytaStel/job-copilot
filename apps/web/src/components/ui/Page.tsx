import type { HTMLAttributes, ReactNode } from 'react';
import { cn } from '../../lib/cn';

export function Page({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn('mx-auto flex w-full max-w-[1380px] flex-col gap-6', className)}
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
    return <div className={cn('grid gap-6', className)}>{children}</div>;
  }

  return (
    <div className={cn('grid gap-6 xl:grid-cols-[minmax(0,1fr)_340px]', className)}>
      <div className="min-w-0">{children}</div>
      <aside className="space-y-4">{aside}</aside>
    </div>
  );
}
