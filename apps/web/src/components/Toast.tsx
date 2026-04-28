import { CheckCircle2, Info, TriangleAlert, X } from 'lucide-react';

import { cn } from '../lib/cn';
import type { ToastMessage } from '../context/ToastContext';

const TOAST_ICON = {
  success: CheckCircle2,
  error: TriangleAlert,
  info: Info,
} satisfies Record<ToastMessage['type'], typeof CheckCircle2>;

const TOAST_CLASS = {
  success: 'border-emerald-400/30 bg-emerald-500/15 text-emerald-100',
  error: 'border-red-400/30 bg-red-500/15 text-red-100',
  info: 'border-primary/30 bg-primary/15 text-foreground',
} satisfies Record<ToastMessage['type'], string>;

export function Toast({
  toast,
  onDismiss,
}: {
  toast: ToastMessage;
  onDismiss: (id: string) => void;
}) {
  const Icon = TOAST_ICON[toast.type];

  return (
    <div
      role={toast.type === 'error' ? 'alert' : 'status'}
      className={cn(
        'pointer-events-auto flex w-full max-w-sm items-start gap-3 rounded-2xl border px-4 py-3 text-sm shadow-2xl shadow-black/30 backdrop-blur-md',
        TOAST_CLASS[toast.type],
      )}
    >
      <Icon className="mt-0.5 h-4 w-4 shrink-0" />

      <div className="min-w-0 flex-1">
        <p className="m-0 font-semibold">{toast.message}</p>
        {toast.description ? (
          <p className="m-0 mt-1 text-xs opacity-80">{toast.description}</p>
        ) : null}
        {toast.action ? (
          <button
            type="button"
            onClick={() => {
              toast.action?.onClick();
              onDismiss(toast.id);
            }}
            className="mt-2 rounded-lg border border-current/30 px-2.5 py-1 text-xs font-semibold transition hover:bg-white/10"
          >
            {toast.action.label}
          </button>
        ) : null}
      </div>

      <button
        type="button"
        onClick={() => onDismiss(toast.id)}
        className="rounded-lg p-1 opacity-70 transition hover:bg-white/10 hover:opacity-100"
        aria-label="Dismiss notification"
      >
        <X className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}
