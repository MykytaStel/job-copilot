/* eslint-disable react-refresh/only-export-components */

import type { ReactNode } from 'react';

import type { BehaviorSignalCount } from '../../api/analytics';
import { EmptyState } from '../../components/ui/EmptyState';
import { HeroMetric } from '../../components/ui/HeroMetric';
import { SectionCard as Section } from '../../components/ui/SectionCard';
import {
  semanticBadgeClass,
  semanticFillClass,
  semanticTextClass,
  type SemanticTone,
} from '../../components/ui/semanticTone';
import { cn } from '../../lib/cn';

export { HeroMetric, Section };

export type Tone = Extract<SemanticTone, 'primary' | 'success' | 'warning' | 'danger' | 'muted'>;

export const toneClasses: Record<Tone, string> = {
  primary: semanticBadgeClass.primary,
  success: semanticBadgeClass.success,
  warning: semanticBadgeClass.warning,
  danger: semanticBadgeClass.danger,
  muted: semanticBadgeClass.muted,
};

export const barToneClasses: Record<Tone, string> = {
  primary: semanticFillClass.primary,
  success: semanticFillClass.success,
  warning: semanticFillClass.warning,
  danger: semanticFillClass.danger,
  muted: semanticFillClass.muted,
};

export function BarList({
  items,
  emptyMessage,
}: {
  items: { label: string; value: number; tone?: Tone }[];
  emptyMessage: string;
}) {
  const maxValue = Math.max(...items.map((item) => item.value), 1);

  if (items.length === 0) {
    return <EmptyState message={emptyMessage} className="px-4 py-4 text-left" />;
  }

  return (
    <div className="space-y-3">
      {items.map((item) => {
        const width = `${Math.round((item.value / maxValue) * 100)}%`;
        const tone = item.tone ?? 'primary';

        return (
          <div key={item.label} className="grid grid-cols-[minmax(0,1fr)_56px] items-center gap-3">
            <div className="min-w-0 space-y-2">
              <div className="flex items-center justify-between gap-3">
                <p className="m-0 truncate text-sm text-card-foreground">{item.label}</p>
                <span className="text-xs text-muted-foreground">{item.value}</span>
              </div>
              <div className="h-2 overflow-hidden rounded-full bg-white-a05">
                <div
                  className={cn(
                    'h-full rounded-full transition-[width] duration-300',
                    barToneClasses[tone],
                  )}
                  style={{ width }}
                />
              </div>
            </div>
            <p className="m-0 text-right text-sm font-semibold text-card-foreground">
              {item.value}
            </p>
          </div>
        );
      })}
    </div>
  );
}

export function ConversionCard({
  label,
  rate,
  numerator,
  denominator,
  tone,
}: {
  label: string;
  rate: number;
  numerator: number;
  denominator: number;
  tone: Tone;
}) {
  const width = `${Math.max(0, Math.min(rate, 1)) * 100}%`;

  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
      <div className="mb-2 flex items-center justify-between gap-3">
        <p className="m-0 text-xs text-muted-foreground">{label}</p>
        <span className={cn('text-lg font-bold', semanticTextClass[tone])}>
          {Math.round(rate * 100)}%
        </span>
      </div>
      <div className="h-2 overflow-hidden rounded-full bg-white-a05">
        <div
          className={cn(
            'h-full rounded-full transition-[width] duration-300',
            barToneClasses[tone],
          )}
          style={{ width }}
        />
      </div>
      <p className="m-0 mt-2 text-xs text-muted-foreground">
        {numerator} / {denominator || 0}
      </p>
    </div>
  );
}

export function SignalList({
  title,
  description,
  items,
  tone,
}: {
  title: string;
  description: string;
  items: BehaviorSignalCount[];
  tone: Tone;
}) {
  if (items.length === 0) {
    return (
      <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
        <p className="m-0 text-sm font-semibold text-card-foreground">{title}</p>
        <p className="m-0 mt-1 text-xs leading-6 text-muted-foreground">{description}</p>
        <EmptyState message="No signal data yet." className="px-4 py-4 text-left" />
      </div>
    );
  }

  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
      <p className="m-0 text-sm font-semibold text-card-foreground">{title}</p>
      <p className="m-0 mt-1 text-xs leading-6 text-muted-foreground">{description}</p>
      <div className="mt-4 space-y-3">
        {items.slice(0, 6).map((item) => (
          <div
            key={item.key}
            className="flex items-start justify-between gap-3 rounded-xl border border-border/60 bg-background/60 px-3 py-3"
          >
            <div className="min-w-0">
              <p className="m-0 truncate text-sm font-medium text-card-foreground">{item.key}</p>
              <div className="mt-2 flex flex-wrap gap-2">
                <span className="rounded-full border border-border bg-white-a04 px-2 py-1 text-[11px] text-muted-foreground">
                  saves {item.saveCount}
                </span>
                <span className="rounded-full border border-border bg-white-a04 px-2 py-1 text-[11px] text-muted-foreground">
                  applies {item.applicationCreatedCount}
                </span>
                <span className="rounded-full border border-border bg-white-a04 px-2 py-1 text-[11px] text-muted-foreground">
                  bad fit {item.badFitCount}
                </span>
              </div>
            </div>
            <span
              className={cn(
                'shrink-0 rounded-full border px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.14em]',
                toneClasses[tone],
              )}
            >
              net {item.netScore}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function PillCloud({
  title,
  items,
  emptyMessage,
  tone,
}: {
  title: string;
  items: string[];
  emptyMessage: string;
  tone: Tone;
}) {
  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
      <p className="m-0 text-sm font-semibold text-card-foreground">{title}</p>
      {items.length === 0 ? (
        <EmptyState message={emptyMessage} className="px-4 py-4 text-left" />
      ) : (
        <div className="mt-4 flex flex-wrap gap-2">
          {items.map((item) => (
            <span
              key={item}
              className={cn(
                'inline-flex items-center rounded-full border px-3 py-1.5 text-xs',
                toneClasses[tone],
              )}
            >
              {item}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

export function TextList({
  title,
  items,
  emptyMessage,
  tone,
}: {
  title: string;
  items: string[];
  emptyMessage: string;
  tone: Tone;
}) {
  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
      <p className="m-0 text-sm font-semibold text-card-foreground">{title}</p>
      {items.length === 0 ? (
        <EmptyState message={emptyMessage} className="px-4 py-4 text-left" />
      ) : (
        <div className="mt-4 space-y-3">
          {items.map((item) => (
            <div
              key={item}
              className="flex items-start gap-3 rounded-xl border border-border/60 bg-background/60 px-3 py-3"
            >
              <span className={cn('mt-1 h-2 w-2 shrink-0 rounded-full', barToneClasses[tone])} />
              <p className="m-0 text-sm leading-6 text-card-foreground">{item}</p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
