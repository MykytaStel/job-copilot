import { Link } from 'react-router-dom';
import { ChevronRight, Lightbulb, Sparkles, Target, TrendingUp } from 'lucide-react';
import { cn } from '../../lib/cn';
import { Badge } from './Badge';
import { Button } from './Button';
import { Card, CardContent, CardHeader, CardTitle } from './Card';

type InsightType = 'tip' | 'recommendation' | 'trend';

export interface AIInsight {
  id: string;
  type: InsightType;
  title: string;
  description: string;
  action?: {
    label: string;
    href: string;
  };
}

const insightIcons = {
  tip: Lightbulb,
  recommendation: Target,
  trend: TrendingUp,
} satisfies Record<InsightType, typeof Lightbulb>;

const insightColors = {
  tip: 'bg-fit-fair/15 text-fit-fair border-fit-fair/30',
  recommendation: 'bg-primary/15 text-primary border-primary/30',
  trend: 'bg-fit-good/15 text-fit-good border-fit-good/30',
} satisfies Record<InsightType, string>;

export function AIInsightPanel({
  insights,
  title = 'AI Insights',
  className,
}: {
  insights: AIInsight[];
  title?: string;
  className?: string;
}) {
  return (
    <Card className={cn('border-border bg-card', className)}>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between gap-3">
          <CardTitle className="flex items-center gap-2 text-base font-semibold">
            <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-primary/15">
              <Sparkles className="h-4 w-4 text-primary" />
            </div>
            {title}
          </CardTitle>
          <Badge
            variant="muted"
            className="rounded-full px-2 py-1 text-[10px] uppercase tracking-wide"
          >
            {insights.length} items
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        {insights.map((insight) => {
          const Icon = insightIcons[insight.type];

          return (
            <div
              key={insight.id}
              className="flex items-start gap-3 rounded-2xl border border-border/70 bg-surface-elevated/40 p-3.5"
            >
              <div
                className={cn(
                  'flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border',
                  insightColors[insight.type],
                )}
              >
                <Icon className="h-4 w-4" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="m-0 text-sm font-medium text-card-foreground">{insight.title}</p>
                <p className="m-0 mt-1 text-xs leading-6 text-muted-foreground">
                  {insight.description}
                </p>
                {insight.action && (
                  <Link to={insight.action.href} className="mt-2 inline-block no-underline">
                    <Button
                      variant="ghost"
                      size="sm"
                      className="px-0 text-xs text-primary hover:text-primary"
                    >
                      {insight.action.label}
                      <ChevronRight className="ml-1 h-3 w-3" />
                    </Button>
                  </Link>
                )}
              </div>
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
}
