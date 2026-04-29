import { useState } from 'react';
import { Star } from 'lucide-react';

import { cn } from '../lib/cn';

type StarRatingProps = {
  value?: number | null;
  disabled?: boolean;
  onChange: (rating: number) => void;
  className?: string;
};

export function StarRating({ value, disabled = false, onChange, className }: StarRatingProps) {
  const [hoveredRating, setHoveredRating] = useState<number | null>(null);
  const activeRating = hoveredRating ?? value ?? 0;

  return (
    <div
      className={cn('inline-flex items-center gap-1', className)}
      onMouseLeave={() => setHoveredRating(null)}
    >
      {[1, 2, 3, 4, 5].map((rating) => {
        const filled = rating <= activeRating;
        return (
          <button
            key={rating}
            type="button"
            title={`${rating} star${rating === 1 ? '' : 's'}`}
            aria-label={`Rate ${rating} out of 5`}
            aria-pressed={value === rating}
            disabled={disabled}
            onMouseEnter={() => setHoveredRating(rating)}
            onFocus={() => setHoveredRating(rating)}
            onBlur={() => setHoveredRating(null)}
            onClick={() => onChange(rating)}
            className="flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-surface-hover focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
          >
            <Star
              className={cn(
                'h-5 w-5 transition-colors',
                filled ? 'fill-amber-400 text-amber-500' : 'text-muted-foreground',
              )}
            />
          </button>
        );
      })}
    </div>
  );
}
