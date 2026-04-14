export function formatDate(
  value: string,
  locale: string = 'en-GB',
  options: Intl.DateTimeFormatOptions = {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  },
) {
  return new Date(value).toLocaleDateString(locale, options);
}

export function formatOptionalDate(
  value?: string,
  locale: string = 'uk-UA',
  options: Intl.DateTimeFormatOptions = {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  },
) {
  if (!value) return null;

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return null;

  return date.toLocaleDateString(locale, options);
}

export function normalizeDateInput(value?: string) {
  return value?.slice(0, 10) ?? '';
}

export function parseOptionalNumber(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return undefined;

  const parsed = Number(trimmed);
  return Number.isFinite(parsed) ? parsed : undefined;
}

export function formatEnumLabel(value: string) {
  return value.replaceAll('_', ' ');
}

export function formatFallbackLabel(value: string) {
  return value
    .split('_')
    .map((part) => {
      if (part.length <= 2) {
        return part.toUpperCase();
      }

      return part.charAt(0).toUpperCase() + part.slice(1);
    })
    .join(' ');
}
