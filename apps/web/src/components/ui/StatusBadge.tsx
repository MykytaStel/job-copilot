import { cn } from '../../lib/cn';
import { formatEnumLabel } from '../../lib/format';
import { Badge } from './Badge';

const STATUS_VARIANTS: Record<
  string,
  'default' | 'info' | 'success' | 'danger' | 'warning' | 'muted'
> = {
  saved: 'muted',
  applied: 'default',
  interview: 'info',
  offer: 'success',
  rejected: 'danger',
  hidden: 'muted',
  'bad fit': 'danger',
  whitelist: 'success',
  blacklist: 'danger',
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
    <Badge
      variant={STATUS_VARIANTS[normalized] ?? 'muted'}
      className={cn('px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em]', className)}
    >
      {label ?? formatEnumLabel(normalized)}
    </Badge>
  );
}
