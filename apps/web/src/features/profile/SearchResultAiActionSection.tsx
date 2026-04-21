import type { ReactNode } from 'react';
import { Sparkles } from 'lucide-react';

import { Button } from '../../components/ui/Button';
import { renderAiErrorMessage } from './profileAiAssist.utils';

export function SearchResultAiActionSection({
  label,
  actionLabel,
  refreshLabel,
  pendingLabel,
  isPending,
  hasContent,
  disabled,
  onAction,
  error,
  errorFallback,
  children,
}: {
  label: string;
  actionLabel: string;
  refreshLabel: string;
  pendingLabel: string;
  isPending: boolean;
  hasContent: boolean;
  disabled: boolean;
  onAction: () => void;
  error: unknown;
  errorFallback: string;
  children?: ReactNode;
}) {
  const hasVisibleState = hasContent || isPending || Boolean(error);

  return (
    <>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: 12,
          marginBottom: hasVisibleState ? 10 : 0,
        }}
      >
        <span className="detailLabel">{label}</span>
        <Button type="button" variant="ghost" size="sm" disabled={disabled} onClick={onAction}>
          <Sparkles size={13} />
          {isPending ? pendingLabel : hasContent ? refreshLabel : actionLabel}
        </Button>
      </div>

      {error && (
        <p className="error" style={{ marginBottom: 0 }}>
          {renderAiErrorMessage(error, errorFallback)}
        </p>
      )}

      {children}
    </>
  );
}
