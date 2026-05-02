import { SurfaceMetric } from '../../components/ui/Surface';

export function SettingsSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="space-y-6">
      <div className="border-b border-border pb-4">
        <h2 className="text-base font-semibold text-foreground">{title}</h2>
      </div>
      {children}
    </div>
  );
}

export function SettingRow({ label, value }: { label: string; value: string }) {
  return (
    <SurfaceMetric>
      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">{label}</p>
      <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">{value}</p>
    </SurfaceMetric>
  );
}
