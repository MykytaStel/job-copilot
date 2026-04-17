import { cn } from '../../lib/cn';
import { formatEnumLabel } from '../../lib/format';

const STATUS_CLASSES: Record<string, string> = {
  saved: 'bg-white/8 text-muted-foreground border-white/10',
  applied: 'bg-primary/15 text-primary border-primary/25',
  interview: 'bg-fit-good/15 text-fit-good border-fit-good/25',
  offer: 'bg-fit-excellent/15 text-fit-excellent border-fit-excellent/25',
  rejected: 'bg-destructive/15 text-destructive border-destructive/25',
  hidden: 'bg-white/6 text-muted-foreground border-white/10',
  'bad fit': 'bg-destructive/15 text-destructive border-destructive/25',
  whitelist: 'bg-fit-excellent/15 text-fit-excellent border-fit-excellent/25',
  blacklist: 'bg-destructive/15 text-destructive border-destructive/25',
};

export function StatusBadge({
  status,
  label,
  className,
}: {
  status: string;
  label?: string;
  className?: string;
}) {
  const normalized = status.replaceAll('_', ' ').toLowerCase();

  return (
    <span
      className={cn(
        'inline-flex items-center rounded-full border px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em]',
        STATUS_CLASSES[normalized] ?? 'bg-secondary/70 text-foreground border-border',
        className,
      )}
    >
      {label ?? formatEnumLabel(normalized)}
    </span>
  );
}
