import type { LucideIcon } from 'lucide-react';
import { cn } from '../../lib/cn';
import { Card, CardContent } from './Card';

interface StatCardProps {
  title: string;
  value: string | number;
  description?: string;
  icon?: LucideIcon;
  trend?: { value: number; label: string };
  className?: string;
}

export function StatCard({ title, value, description, icon: Icon, trend, className }: StatCardProps) {
  return (
    <Card className={cn('border-border bg-card', className)}>
      <CardContent className="px-5">
        <div className="flex items-start justify-between">
          <div className="space-y-1.5">
            <p className="text-sm font-medium text-muted-foreground">{title}</p>
            <p className="text-2xl font-bold text-card-foreground">{value}</p>
            {description && (
              <p className="text-sm text-muted-foreground">{description}</p>
            )}
            {trend && (
              <div className="mt-1 flex items-center gap-1.5">
                <span
                  className={cn(
                    'text-xs font-medium',
                    trend.value >= 0 ? 'text-fit-excellent' : 'text-fit-poor',
                  )}
                >
                  {trend.value >= 0 ? '+' : ''}{trend.value}%
                </span>
                <span className="text-xs text-muted-foreground">{trend.label}</span>
              </div>
            )}
          </div>
          {Icon && (
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
              <Icon className="h-5 w-5 text-primary" />
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

export function AnalyticsCard(props: StatCardProps) {
  const { title, value, description, icon: Icon, trend, className } = props;

  return (
    <Card className={cn('border-border bg-card', className)}>
      <CardContent className="px-5">
        <div className="mb-4 flex items-center justify-between">
          <p className="text-sm font-medium text-muted-foreground">{title}</p>
          {Icon && (
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/10">
              <Icon className="h-4 w-4 text-primary" />
            </div>
          )}
        </div>
        <div className="space-y-1.5">
          <p className="text-3xl font-bold text-card-foreground">{value}</p>
          {trend && (
            <div className="flex items-center gap-2">
              <span
                className={cn(
                  'text-sm font-medium',
                  trend.value >= 0 ? 'text-fit-excellent' : 'text-fit-poor',
                )}
              >
                {trend.value >= 0 ? '+' : ''}
                {trend.value}%
              </span>
              <span className="text-sm text-muted-foreground">{trend.label}</span>
            </div>
          )}
          {description && <p className="pt-1 text-sm text-muted-foreground">{description}</p>}
        </div>
      </CardContent>
    </Card>
  );
}
