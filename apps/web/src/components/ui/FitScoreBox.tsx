import { cn } from '../../lib/cn';

export type FitBand = 'excellent' | 'good' | 'fair' | 'poor';

export function getFitBand(score: number): FitBand {
  if (score >= 85) return 'excellent';
  if (score >= 70) return 'good';
  if (score >= 50) return 'fair';
  return 'poor';
}

export function getFitLabel(band: FitBand): string {
  const labels: Record<FitBand, string> = {
    excellent: 'Excellent Match',
    good: 'Good Match',
    fair: 'Fair Match',
    poor: 'Weak Match',
  };
  return labels[band];
}

const bandClasses: Record<FitBand, { bg: string; text: string; ring: string; bar: string }> = {
  excellent: {
    bg: 'bg-fit-excellent/15',
    text: 'text-fit-excellent',
    ring: 'ring-fit-excellent/30',
    bar: 'bg-fit-excellent',
  },
  good: {
    bg: 'bg-fit-good/15',
    text: 'text-fit-good',
    ring: 'ring-fit-good/30',
    bar: 'bg-fit-good',
  },
  fair: {
    bg: 'bg-fit-fair/15',
    text: 'text-fit-fair',
    ring: 'ring-fit-fair/30',
    bar: 'bg-fit-fair',
  },
  poor: {
    bg: 'bg-fit-poor/15',
    text: 'text-fit-poor',
    ring: 'ring-fit-poor/30',
    bar: 'bg-fit-poor',
  },
};

export interface FitScoreBoxProps {
  score: number;
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
  className?: string;
}

const sizeClasses = {
  sm: 'h-10 w-10 text-sm',
  md: 'h-14 w-14 text-lg',
  lg: 'h-20 w-20 text-2xl',
};

const labelSizeClasses = {
  sm: 'text-[9px]',
  md: 'text-[10px]',
  lg: 'text-xs',
};

/** Badge-style score box (square with rounded corners). */
export function FitScoreBox({
  score,
  size = 'md',
  showLabel = false,
  className,
}: FitScoreBoxProps) {
  const band = getFitBand(score);
  const c = bandClasses[band];

  return (
    <div className={cn('flex flex-col items-center gap-1', className)}>
      <div
        className={cn(
          'flex items-center justify-center rounded-xl ring-1',
          c.bg,
          c.text,
          c.ring,
          sizeClasses[size],
        )}
      >
        <span className="font-bold">{score}</span>
      </div>
      {showLabel && (
        <span className={cn('font-medium', c.text, labelSizeClasses[size])}>
          {getFitLabel(band)}
        </span>
      )}
    </div>
  );
}

/** Circular SVG progress variant. */
export function FitScoreCircular({
  score,
  size = 'md',
  showLabel = false,
  className,
}: FitScoreBoxProps) {
  const band = getFitBand(score);
  const c = bandClasses[band];

  const radius = size === 'sm' ? 18 : size === 'md' ? 24 : 32;
  const strokeWidth = size === 'sm' ? 3 : size === 'md' ? 4 : 5;
  const circumference = 2 * Math.PI * radius;
  const progress = (score / 100) * circumference;
  const svgSize = (radius + strokeWidth) * 2;

  const textSizeClasses = { sm: 'text-sm', md: 'text-lg', lg: 'text-2xl' };

  return (
    <div className={cn('flex flex-col items-center gap-1', className)}>
      <div className="relative">
        <svg width={svgSize} height={svgSize} className="transform -rotate-90">
          <circle
            cx={radius + strokeWidth}
            cy={radius + strokeWidth}
            r={radius}
            strokeWidth={strokeWidth}
            fill="none"
            className="stroke-muted"
          />
          <circle
            cx={radius + strokeWidth}
            cy={radius + strokeWidth}
            r={radius}
            strokeWidth={strokeWidth}
            fill="none"
            strokeLinecap="round"
            strokeDasharray={circumference}
            strokeDashoffset={circumference - progress}
            className={cn(
              'transition-all duration-500 ease-out',
              c.text.replace('text-', 'stroke-'),
            )}
          />
        </svg>
        <div className="absolute inset-0 flex items-center justify-center">
          <span className={cn('font-bold', c.text, textSizeClasses[size])}>{score}</span>
        </div>
      </div>
      {showLabel && <span className={cn('font-medium text-xs', c.text)}>{getFitLabel(band)}</span>}
    </div>
  );
}
