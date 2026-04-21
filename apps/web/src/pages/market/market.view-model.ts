import {
  ArrowDownRight,
  ArrowUpRight,
  Minus,
  type LucideIcon,
} from 'lucide-react';

import type {
  MarketTrend,
} from '../../api/market';

const numberFormatter = new Intl.NumberFormat('en-US');
const percentFormatter = new Intl.NumberFormat('en-US', { maximumFractionDigits: 0 });

export function formatCount(value: number) {
  return numberFormatter.format(value);
}

export function formatPercent(value: number) {
  return `${percentFormatter.format(value)}%`;
}

export function formatSalary(value: number) {
  return numberFormatter.format(Math.round(value));
}

export function titleCase(value: string) {
  return value.charAt(0).toUpperCase() + value.slice(1);
}

export function getTrendMeta(trend: MarketTrend | number): {
  icon: LucideIcon;
  label: string;
  className: string;
} {
  if (typeof trend === 'number') {
    if (trend > 0) {
      return {
        icon: ArrowUpRight,
        label: `+${trend}`,
        className: 'border-fit-excellent/25 bg-fit-excellent/10 text-fit-excellent',
      };
    }

    if (trend < 0) {
      return {
        icon: ArrowDownRight,
        label: `${trend}`,
        className: 'border-destructive/25 bg-destructive/10 text-destructive',
      };
    }

    return {
      icon: Minus,
      label: '0',
      className: 'border-border bg-white/[0.04] text-muted-foreground',
    };
  }

  if (trend === 'up') {
    return {
      icon: ArrowUpRight,
      label: 'Up',
      className: 'border-fit-excellent/25 bg-fit-excellent/10 text-fit-excellent',
    };
  }

  if (trend === 'down') {
    return {
      icon: ArrowDownRight,
      label: 'Down',
      className: 'border-destructive/25 bg-destructive/10 text-destructive',
    };
  }

  return {
    icon: Minus,
    label: 'Flat',
    className: 'border-border bg-white/[0.04] text-muted-foreground',
  };
}
