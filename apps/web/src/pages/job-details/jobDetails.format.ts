export function formatSalary(min?: number, max?: number, currency?: string) {
  if (!min && !max) return null;

  const symbol = currency === 'USD' ? '$' : currency === 'EUR' ? '€' : (currency ?? '');
  const formatValue = (value: number) => `${symbol}${value.toLocaleString()}`;

  if (min && max) {
    return `${formatValue(min)} – ${formatValue(max)}`;
  }

  return min ? `від ${formatValue(min)}` : `до ${formatValue(max!)}`;
}
