import { cn } from '../../lib/cn';
import { EmptyState } from './EmptyState';

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
      ? 'bg-fit-excellent/12 text-fit-excellent border-fit-excellent/15'
      : tone === 'danger'
        ? 'bg-destructive/12 text-destructive border-destructive/15'
        : 'bg-secondary/70 text-secondary-foreground border-border';

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
