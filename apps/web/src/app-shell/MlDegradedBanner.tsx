import { X, TriangleAlert } from 'lucide-react';

interface Props {
  onDismiss: () => void;
}

export function MlDegradedBanner({ onDismiss }: Props) {
  return (
    <div className="flex items-center gap-3 rounded-[var(--radius-md)] border border-amber-500/25 bg-amber-500/8 px-4 py-2.5 text-sm text-amber-300 mb-4">
      <TriangleAlert className="h-4 w-4 shrink-0 text-amber-400" />
      <span className="flex-1">
        AI enrichment is currently unavailable. Ranking and fit explanations use the deterministic
        baseline only.
      </span>
      <button
        onClick={onDismiss}
        className="ml-2 rounded p-0.5 text-amber-400/70 hover:text-amber-300 focus:outline-none"
        aria-label="Dismiss"
      >
        <X className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}
