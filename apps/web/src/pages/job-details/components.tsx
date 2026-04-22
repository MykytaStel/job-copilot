import type { ComponentProps, ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';

import { AccentIconFrame } from '../../components/ui/AccentIconFrame';
import { Button } from '../../components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/Card';
import { cn } from '../../lib/cn';

export function Section({
  title,
  description,
  icon: Icon,
  children,
}: {
  title: string;
  description: string;
  icon: LucideIcon;
  children: ReactNode;
}) {
  return (
    <Card className="border-border bg-card">
      <CardHeader className="gap-3">
        <div className="flex items-start gap-3">
          <AccentIconFrame size="lg">
            <Icon className="h-5 w-5" />
          </AccentIconFrame>
          <div>
            <CardTitle className="text-base font-semibold">{title}</CardTitle>
            <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">{description}</p>
          </div>
        </div>
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
}

export function HeroMetric({
  label,
  value,
  icon: Icon,
}: {
  label: string;
  value: string | number;
  icon: LucideIcon;
}) {
  return (
    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
      <div className="flex items-center gap-3">
        <AccentIconFrame size="md">
          <Icon className="h-4 w-4" />
        </AccentIconFrame>
        <div>
          <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
            {label}
          </p>
          <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">{value}</p>
        </div>
      </div>
    </div>
  );
}

export function FeedbackButton({ children, className, ...props }: ComponentProps<typeof Button>) {
  return (
    <Button
      variant="outline"
      size="sm"
      className={cn('w-full justify-start', className)}
      {...props}
    >
      {children}
    </Button>
  );
}
