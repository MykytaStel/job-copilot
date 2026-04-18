import { cn } from '../../lib/cn';

type OptionCardGroupItem<T extends string> = {
  id: T;
  label: string;
};

export function OptionCardGroup<T extends string>({
  options,
  value,
  onToggle,
}: {
  options: Array<OptionCardGroupItem<T>>;
  value: T[];
  onToggle: (value: T) => void;
}) {
  return (
    <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
      {options.map((option) => (
        <label
          key={option.id}
          className={cn(
            'flex cursor-pointer items-center gap-3 rounded-2xl border border-border bg-card/70 px-4 py-3.5 text-sm transition-colors',
            value.includes(option.id) && 'border-primary/35 bg-primary/8 text-card-foreground shadow-[inset_0_0_0_1px_rgba(149,167,255,0.1)]',
          )}
        >
          <input
            type="checkbox"
            checked={value.includes(option.id)}
            onChange={() => onToggle(option.id)}
            className="h-4 w-4 accent-[var(--color-primary)]"
          />
          <span className="leading-6">{option.label}</span>
        </label>
      ))}
    </div>
  );
}
