import { cn } from '../../lib/cn';
import { Button } from './Button';

interface FilterOption {
  id: string;
  label: string;
  count?: number;
}

interface FilterChipsProps {
  options: FilterOption[];
  selected: string[];
  onChange: (selected: string[]) => void;
  multiple?: boolean;
  className?: string;
}

export function FilterChips({
  options,
  selected,
  onChange,
  multiple = false,
  className,
}: FilterChipsProps) {
  function handleSelect(id: string) {
    if (multiple) {
      onChange(selected.includes(id) ? selected.filter((s) => s !== id) : [...selected, id]);
    } else {
      onChange(selected.includes(id) && selected.length === 1 ? [] : [id]);
    }
  }

  return (
    <div className={cn('flex flex-wrap gap-2.5', className)}>
      {options.map((option) => {
        const isActive = selected.includes(option.id);
        return (
          <Button
            key={option.id}
            type="button"
            variant="outline"
            active={isActive}
            onClick={() => handleSelect(option.id)}
            className={cn(
              'h-9 rounded-full px-3.5 text-xs font-medium shadow-none',
              isActive
                ? 'border-primary bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground'
                : 'border-border bg-white/[0.03] text-muted-foreground hover:bg-secondary hover:text-foreground',
            )}
          >
            {option.label}
            {option.count !== undefined && (
              <span
                className={cn(
                  'ml-1 rounded-full px-1.5 py-0.5 text-[10px] font-medium',
                  isActive ? 'bg-white/20 text-white' : 'bg-white/10 text-content-muted',
                )}
              >
                {option.count}
              </span>
            )}
          </Button>
        );
      })}
    </div>
  );
}
