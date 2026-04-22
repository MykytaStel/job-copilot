import {
  ArrowDownRight,
  ArrowUpRight,
  Minus,
  type LucideIcon,
} from 'lucide-react';
import { semanticBadgeClass } from '../../components/ui/semanticTone';

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
        className: semanticBadgeClass.success,
      };
    }

    if (trend < 0) {
      return {
        icon: ArrowDownRight,
        label: `${trend}`,
        className: semanticBadgeClass.danger,
      };
    }

    return {
      icon: Minus,
      label: '0',
      className: semanticBadgeClass.muted,
    };
  }

  if (trend === 'up') {
    return {
      icon: ArrowUpRight,
      label: 'Up',
      className: semanticBadgeClass.success,
    };
  }

  if (trend === 'down') {
    return {
      icon: ArrowDownRight,
      label: 'Down',
      className: semanticBadgeClass.danger,
    };
  }

  return {
    icon: Minus,
    label: 'Flat',
    className: semanticBadgeClass.muted,
  };
}
