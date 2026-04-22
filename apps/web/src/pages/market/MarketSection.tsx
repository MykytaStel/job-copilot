import type { ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';

import { AccentIconFrame } from '../../components/ui/AccentIconFrame';
import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/Card';

export function MarketSection({
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
