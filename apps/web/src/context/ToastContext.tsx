import {
  createContext,
  createElement,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import { createPortal } from 'react-dom';

import { Toast } from '../components/Toast';

export type ToastType = 'success' | 'error' | 'info';

export type ToastMessage = {
  id: string;
  type: ToastType;
  message: string;
  description?: string;
  action?: {
    label: string;
    onClick: () => void;
  };
};

type ShowToastInput = {
  type?: ToastType;
  message: string;
  description?: string;
  action?: {
    label: string;
    onClick: () => void;
  };
  durationMs?: number;
};

type ToastContextValue = {
  showToast: (toast: ShowToastInput) => string;
  dismissToast: (id: string) => void;
};

const ToastContext = createContext<ToastContextValue | null>(null);

const AUTO_DISMISS_MS = 3_000;
const MAX_VISIBLE_TOASTS = 3;

function createToastId() {
  return `toast_${Date.now()}_${Math.random().toString(36).slice(2)}`;
}

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastMessage[]>([]);

  const dismissToast = useCallback((id: string) => {
    setToasts((current) => current.filter((toast) => toast.id !== id));
  }, []);

  const showToast = useCallback(
    ({ type = 'info', message, description, action, durationMs }: ShowToastInput) => {
      const id = createToastId();

      setToasts((current) => [
        { id, type, message, description, action },
        ...current,
      ].slice(0, MAX_VISIBLE_TOASTS));

      window.setTimeout(() => dismissToast(id), durationMs ?? AUTO_DISMISS_MS);

      return id;
    },
    [dismissToast],
  );

  const value = useMemo(
    () => ({ showToast, dismissToast }),
    [showToast, dismissToast],
  );

  return (
    <ToastContext.Provider value={value}>
      {children}

      {typeof document !== 'undefined'
        ? createPortal(
            <div className="pointer-events-none fixed right-4 top-4 z-[100] flex w-[calc(100vw-2rem)] max-w-sm flex-col gap-3 sm:right-6 sm:top-6">
              {toasts.map((toast) => (
                <Toast key={toast.id} toast={toast} onDismiss={dismissToast} />
              ))}
            </div>,
            document.body,
          )
        : null}
    </ToastContext.Provider>
  );
}

export function useToast() {
  const value = useContext(ToastContext);

  if (!value) {
    throw new Error('useToast must be used inside ToastProvider');
  }

  return value;
}
