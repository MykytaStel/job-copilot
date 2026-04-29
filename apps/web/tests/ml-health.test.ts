import { describe, expect, it } from 'vitest';

import { isMlDegraded } from '../src/api/ml-health';

describe('ml health api', () => {
  it('handles current component-based /ready payloads without checks', () => {
    expect(
      isMlDegraded({
        status: 'degraded',
        components: {
          database: { status: 'ok', latency_ms: 8 },
          ml_sidecar: { status: 'ok' },
          ingestion: { status: 'stale', last_run_at: null },
        },
      }),
    ).toBe(true);
  });

  it('keeps supporting legacy check-based degraded payloads', () => {
    expect(
      isMlDegraded({
        status: 'ready',
        service: 'ml',
        checks: [{ name: 'enrichment_provider', status: 'degraded' }],
      }),
    ).toBe(true);
  });

  it('treats a ready component payload as healthy', () => {
    expect(
      isMlDegraded({
        status: 'ready',
        components: {
          database: { status: 'ok', latency_ms: 4 },
          ml_sidecar: { status: 'ok' },
          ingestion: { status: 'ok', last_run_at: '2026-04-29T10:00:00Z' },
        },
      }),
    ).toBe(false);
  });
});
