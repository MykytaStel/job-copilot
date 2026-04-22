import type { ComponentProps } from 'react';

import { Button } from '../../components/ui/Button';
import { cn } from '../../lib/cn';

export { HeroMetric } from '../../components/ui/HeroMetric';
export { SectionCard as Section } from '../../components/ui/SectionCard';

export function FeedbackButton({ children, className, ...props }: ComponentProps<typeof Button>) {
  return (
    <Button
      variant="outline"
      size="sm"
      className={cn('w-full justify-start', className)}
      {...props}
    >
      {children}
    </Button>
  );
}
