import type { ApplicationDetail } from '@job-copilot/shared';

export function formatCompensation(detail: ApplicationDetail) {
  const offer = detail.offer;
  if (!offer) return null;

  const min = offer.compensationMin;
  const max = offer.compensationMax;
  const currency = offer.compensationCurrency ?? '';

  if (min == null && max == null) return null;
  if (min != null && max != null) return `${min} - ${max} ${currency}`.trim();
  if (min != null) return `${min}+ ${currency}`.trim();
  return `Up to ${max} ${currency}`.trim();
}
