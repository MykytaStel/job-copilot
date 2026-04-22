import type { LucideIcon } from 'lucide-react';
import { AccentIconFrame } from './AccentIconFrame';

interface HeroMetricProps {
  label: string;
  value: string | number;
  icon: LucideIcon;
}

export function HeroMetric({ label, value, icon: Icon }: HeroMetricProps) {
  return (
    <div className="rounded-2xl border border-border/70 bg-white-a04 px-4 py-3">
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
