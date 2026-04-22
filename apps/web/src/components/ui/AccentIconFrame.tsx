import type { HTMLAttributes } from 'react';
import { cn } from '../../lib/cn';
import { semanticIconFrameClass, type SemanticTone } from './semanticTone';

type AccentIconFrameSize = 'sm' | 'md' | 'lg' | 'xl';

const sizeClass: Record<AccentIconFrameSize, string> = {
  sm: 'h-8 w-8 rounded-lg',
  md: 'h-10 w-10 rounded-xl',
  lg: 'h-11 w-11 rounded-2xl',
  xl: 'h-16 w-16 rounded-2xl',
};

export function AccentIconFrame({
  size = 'md',
  tone = 'primary',
  className,
  children,
  ...props
}: HTMLAttributes<HTMLDivElement> & { size?: AccentIconFrameSize; tone?: SemanticTone }) {
  return (
    <div
      className={cn(
        'flex shrink-0 items-center justify-center border',
        semanticIconFrameClass[tone],
        sizeClass[size],
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
}
