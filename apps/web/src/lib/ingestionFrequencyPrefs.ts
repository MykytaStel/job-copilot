export type IngestionFrequencyMinutes = 30 | 60 | 120;

export const INGESTION_FREQUENCY_OPTIONS: IngestionFrequencyMinutes[] = [
  30,
  60,
  120,
];

const INGESTION_FREQUENCY_KEY = 'jc_ingestion_frequency_minutes';
const DEFAULT_INGESTION_FREQUENCY: IngestionFrequencyMinutes = 60;

function isIngestionFrequencyMinutes(
  value: number,
): value is IngestionFrequencyMinutes {
  return value === 30 || value === 60 || value === 120;
}

export function readIngestionFrequency(): IngestionFrequencyMinutes {
  try {
    const raw = window.localStorage.getItem(INGESTION_FREQUENCY_KEY);
    if (!raw) return DEFAULT_INGESTION_FREQUENCY;

    const parsed = Number(raw);
    return isIngestionFrequencyMinutes(parsed)
      ? parsed
      : DEFAULT_INGESTION_FREQUENCY;
  } catch {
    return DEFAULT_INGESTION_FREQUENCY;
  }
}

export function writeIngestionFrequency(
  value: IngestionFrequencyMinutes,
): void {
  try {
    window.localStorage.setItem(INGESTION_FREQUENCY_KEY, String(value));
  } catch {
    // Ignore localStorage write errors.
  }
}