import type { ErrorInfo, ReactNode } from 'react';
import { Component } from 'react';

type ErrorBoundaryProps = {
  children: ReactNode;
  fallback?: ReactNode;
};

type ErrorBoundaryState = {
  hasError: boolean;
};

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = {
    hasError: false,
  };

  static getDerivedStateFromError(): ErrorBoundaryState {
    return {
      hasError: true,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    if (import.meta.env.DEV) {
      console.error('Route ErrorBoundary caught an error:', error, errorInfo);
    }
  }

  render() {
    if (this.state.hasError) {
      return (
        this.props.fallback ?? (
          <section
            style={{
              minHeight: '60vh',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              padding: '24px',
            }}
          >
            <div
              style={{
                width: '100%',
                maxWidth: 520,
                borderRadius: 24,
                border: '1px solid var(--color-border-soft)',
                background: 'rgba(18, 25, 39, 0.82)',
                boxShadow: '0 24px 80px rgba(0, 0, 0, 0.28)',
                padding: 24,
                color: 'var(--color-text-primary)',
              }}
            >
              <p
                style={{
                  margin: '0 0 8px',
                  fontSize: 18,
                  fontWeight: 700,
                }}
              >
                Something went wrong.
              </p>

              <p
                style={{
                  margin: 0,
                  color: 'var(--color-text-secondary)',
                  lineHeight: 1.5,
                }}
              >
                Reload page.
              </p>

              <button
                type="button"
                onClick={() => window.location.reload()}
                style={{
                  marginTop: 18,
                  border: '1px solid var(--color-border-soft)',
                  borderRadius: 999,
                  background: 'rgba(255, 255, 255, 0.08)',
                  color: 'var(--color-text-primary)',
                  cursor: 'pointer',
                  padding: '10px 16px',
                  fontWeight: 700,
                }}
              >
                Reload page
              </button>
            </div>
          </section>
        )
      );
    }

    return this.props.children;
  }
}

export function withErrorBoundary(children: ReactNode) {
  return <ErrorBoundary>{children}</ErrorBoundary>;
}