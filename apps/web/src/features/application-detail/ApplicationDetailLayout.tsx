import type { ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';

import { SectionHeader } from '../../components/ui/SectionHeader';
import { SurfaceInset, SurfaceMetric, SurfaceSection } from '../../components/ui/Surface';

export function Panel({
  title,
  description,
  icon,
  children,
}: {
  title: string;
  description: string;
  icon: LucideIcon;
  children: ReactNode;
}) {
  return (
    <SurfaceSection>
      <SectionHeader title={title} description={description} icon={icon} />
      {children}
    </SurfaceSection>
  );
}

export function InnerPanel({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <SurfaceInset className="space-y-4">
      <div>
        <p className="m-0 text-sm font-semibold text-card-foreground">{title}</p>
        {description ? (
          <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">{description}</p>
        ) : null}
      </div>
      {children}
    </SurfaceInset>
  );
}

export function SummaryMetric({ label, value }: { label: string; value: string | number }) {
  return (
    <SurfaceMetric>
      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">{label}</p>
      <p className="m-0 mt-2 text-sm font-semibold text-card-foreground">{value}</p>
    </SurfaceMetric>
  );
}
