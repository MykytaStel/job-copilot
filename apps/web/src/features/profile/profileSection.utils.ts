import { formatFallbackLabel } from '../../lib/format';

export function formatSeniorityLabel(value: string) {
  return value.trim() ? formatFallbackLabel(value) : 'Not specified';
}

export function renderErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback;
}
