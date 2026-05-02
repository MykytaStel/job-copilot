import { cn } from '../../lib/cn';
import type { DensityMode, SortMode } from '../../lib/displayPrefs';
import { useDisplayPrefs } from '../../lib/useDisplayPrefs';
import { SettingsSection } from './settingsShared';

const SORT_OPTIONS: { value: SortMode; label: string }[] = [
  { value: 'relevance', label: 'Relevance' },
  { value: 'date', label: 'Date' },
  { value: 'salary', label: 'Salary' },
];

export function DisplaySettingsSection() {
  const { density, sortMode, setDensity, setSortMode } = useDisplayPrefs();

  return (
    <SettingsSection title="Display">
      <div className="space-y-6">
        <div className="space-y-2">
          <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
            Job card density
          </p>
          <div className="flex flex-wrap gap-2">
            {(['compact', 'normal', 'comfortable'] as DensityMode[]).map((value) => (
              <button
                key={value}
                onClick={() => setDensity(value)}
                className={cn(
                  'rounded-[var(--radius-md)] border px-3 py-1.5 text-sm font-medium capitalize transition-colors',
                  density === value
                    ? 'border-primary bg-primary/10 text-primary'
                    : 'border-border text-muted-foreground hover:border-border/80 hover:text-foreground',
                )}
              >
                {value}
              </button>
            ))}
          </div>
          <p className="text-[11px] text-muted-foreground/70">
            Controls spacing between job cards in the feed.
          </p>
        </div>

        <div className="space-y-2">
          <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
            Default job sort
          </p>
          <div className="flex flex-wrap gap-2">
            {SORT_OPTIONS.map(({ value, label }) => (
              <button
                key={value}
                onClick={() => setSortMode(value)}
                className={cn(
                  'rounded-[var(--radius-md)] border px-3 py-1.5 text-sm font-medium transition-colors',
                  sortMode === value
                    ? 'border-primary bg-primary/10 text-primary'
                    : 'border-border text-muted-foreground hover:border-border/80 hover:text-foreground',
                )}
              >
                {label}
              </button>
            ))}
          </div>
          <p className="text-[11px] text-muted-foreground/70">
            Applied when you open the dashboard. Relevance requires an active profile.
          </p>
        </div>
      </div>
    </SettingsSection>
  );
}
