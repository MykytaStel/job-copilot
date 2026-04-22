import type { ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';
import { AccentIconFrame } from './AccentIconFrame';
import { Card, CardContent, CardHeader, CardTitle } from './Card';
import { cn } from '../../lib/cn';

interface SectionCardProps {
  title: string;
  description?: string;
  icon: LucideIcon;
  eyebrow?: string;
  action?: ReactNode;
  children: ReactNode;
  className?: string;
}

export function SectionCard({
  title,
  description,
  icon: Icon,
  eyebrow,
  action,
  children,
  className,
}: SectionCardProps) {
  return (
    <Card className={cn('border-border bg-card', className)}>
      <CardHeader className="gap-3">
        <div className="flex items-start justify-between gap-4">
          <div className="flex min-w-0 items-start gap-3">
            <AccentIconFrame size="lg">
              <Icon className="h-5 w-5" />
            </AccentIconFrame>
            <div className="min-w-0">
              {eyebrow ? (
                <p className="m-0 text-[11px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
                  {eyebrow}
                </p>
              ) : null}
              <CardTitle className="mt-1 text-base font-semibold">{title}</CardTitle>
              {description ? (
                <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">{description}</p>
              ) : null}
            </div>
          </div>
          {action}
        </div>
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
}
