import type { ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';

import { EmptyState } from '../../components/ui/EmptyState';

import { Section } from './AnalyticsHelpers';

export function AnalyticsEnrichmentSection({
  title,
  description,
  icon,
  eyebrow,
  isLoading,
  error,
  loadingMessage,
  errorMessage,
  emptyMessage,
  children,
}: {
  title: string;
  description: string;
  icon: LucideIcon;
  eyebrow: string;
  isLoading: boolean;
  error: unknown;
  loadingMessage: string;
  errorMessage: string;
  emptyMessage: string;
  children?: ReactNode;
}) {
  return (
    <Section title={title} description={description} icon={icon} eyebrow={eyebrow}>
      {isLoading ? (
        <EmptyState message={loadingMessage} className="px-4 py-4 text-left" />
      ) : error ? (
        <EmptyState
          message={(error as Error).message || errorMessage}
          className="px-4 py-4 text-left"
        />
      ) : children ? (
        children
      ) : (
        <EmptyState message={emptyMessage} className="px-4 py-4 text-left" />
      )}
    </Section>
  );
}
