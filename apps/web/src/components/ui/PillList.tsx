import { cn } from '../../lib/cn';
import { EmptyState } from './EmptyState';
import { semanticBadgeClass } from './semanticTone';

export function PillList({
  items,
  emptyLabel,
  tone = 'default',
}: {
  items: string[];
  emptyLabel: string;
  tone?: 'default' | 'success' | 'danger';
}) {
  if (items.length === 0) {
    return <EmptyState message={emptyLabel} className="px-4 py-4 text-left" />;
  }

  const toneClass =
    tone === 'success'
      ? semanticBadgeClass.success
      : tone === 'danger'
        ? semanticBadgeClass.danger
        : semanticBadgeClass.muted;

  return (
    <div className="flex flex-wrap gap-2">
      {items.map((item) => (
        <span
          key={item}
          className={cn(
            'inline-flex items-center rounded-full border px-3 py-1.5 text-xs font-medium',
            toneClass,
          )}
        >
          {item}
        </span>
      ))}
    </div>
  );
}
