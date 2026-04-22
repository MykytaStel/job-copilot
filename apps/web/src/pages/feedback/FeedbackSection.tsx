import type { ReactNode } from 'react';
import { Badge } from '../../components/ui/Badge';
import { Card, CardContent, CardHeader } from '../../components/ui/Card';

interface FeedbackSectionProps {
  title: string;
  icon: ReactNode;
  description?: string;
  count: number;
  children: ReactNode;
}

export function FeedbackSection({
  title,
  icon,
  description,
  count,
  children,
}: FeedbackSectionProps) {
  return (
    <Card className="border-border bg-card">
      <CardHeader className="gap-3">
        <div className="flex items-center gap-3">
          <span className="flex h-10 w-10 items-center justify-center rounded-xl border border-border bg-white-a04 text-content-muted">
            {icon}
          </span>
          <div>
            <h2 className="m-0 text-[15px] font-semibold text-content">{title}</h2>
            {description ? (
              <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">{description}</p>
            ) : null}
          </div>
          <Badge variant="muted" className="ml-auto rounded-lg px-2 py-0.5 text-xs">
            {count}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">{children}</CardContent>
    </Card>
  );
}
