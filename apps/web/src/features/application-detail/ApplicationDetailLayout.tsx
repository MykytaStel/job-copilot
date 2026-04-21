import type { ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';

import { SectionHeader } from '../../components/ui/SectionHeader';

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
    <section className="space-y-5 rounded-[24px] border border-border bg-card/85 p-7">
      <SectionHeader title={title} description={description} icon={icon} />
      {children}
    </section>
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
    <div className="space-y-4 rounded-2xl border border-border/70 bg-white/[0.03] p-4">
      <div>
        <p className="m-0 text-sm font-semibold text-card-foreground">{title}</p>
        {description ? (
          <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">{description}</p>
        ) : null}
      </div>
      {children}
    </div>
  );
}

export function SummaryMetric({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">{label}</p>
      <p className="m-0 mt-2 text-sm font-semibold text-card-foreground">{value}</p>
    </div>
  );
}
