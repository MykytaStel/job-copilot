import { CheckCircle2 } from 'lucide-react';

import type { ProfileCompletionState } from '../features/profile/profileCompletion';
import { Badge } from './ui/Badge';

export function ProfileCompletion({ completion }: { completion: ProfileCompletionState }) {
  function scrollToTarget(targetId: string) {
    document.getElementById(targetId)?.scrollIntoView({
      behavior: 'smooth',
      block: 'center',
    });
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
            Completion
          </p>
          <div className="mt-1 flex flex-wrap items-center gap-2">
            <p className="m-0 text-sm font-semibold text-card-foreground">
              {completion.percent}% complete
            </p>
            {completion.percent === 100 && (
              <Badge variant="success" className="px-2 py-0.5 text-xs">
                <CheckCircle2 className="h-3.5 w-3.5" />
                Profile complete
              </Badge>
            )}
          </div>
          <p className="m-0 mt-1 text-xs text-muted-foreground">
            {completion.completedWeight}/{completion.totalWeight} completion points ready
          </p>
        </div>
      </div>

      <div
        className="h-2 rounded-full bg-surface-soft"
        role="progressbar"
        aria-label="Profile completion"
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={completion.percent}
      >
        <div
          className="h-2 rounded-full bg-[image:var(--gradient-button)] transition-[width] duration-300"
          style={{ width: `${completion.percent}%` }}
        />
      </div>

      <div>
        <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
          Missing checklist
        </p>
        {completion.missing.length > 0 ? (
          <div className="mt-2 flex flex-wrap gap-2">
            {completion.missing.map((item) => (
              <button
                key={item.id}
                type="button"
                onClick={() => scrollToTarget(item.targetId)}
                className="inline-flex items-center rounded-full border border-border bg-white-a04 px-2.5 py-1 text-left text-xs font-medium text-muted-foreground transition-colors hover:border-primary/40 hover:bg-primary/12 hover:text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/45"
              >
                {item.label}
              </button>
            ))}
          </div>
        ) : (
          <p className="m-0 mt-2 text-xs text-muted-foreground">
            Profile and preferences are ready for ranking and AI flows.
          </p>
        )}
      </div>
    </div>
  );
}
