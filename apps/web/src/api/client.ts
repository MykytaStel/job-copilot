import type { EngineApiError } from './engine-types/health';
import {
  readProfileId,
  resolveProfileId as resolveProfileSessionId,
  withProfileIdQuery as appendProfileIdQuery,
  writeProfileId,
} from '../lib/profileSession';
import { buildAuthHeaders } from '../lib/authSession';

const API_URL = import.meta.env.VITE_ENGINE_API_URL?.trim() || 'http://localhost:8080';
const ML_URL = import.meta.env.VITE_ML_URL?.trim() || 'http://localhost:8000';

export const RECENT_JOBS_LIMIT_MAX = 200;

export async function mlRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${ML_URL}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...init,
  });
  if (!res.ok) {
    const body = (await res.json().catch(() => ({}))) as { detail?: string };
    throw new Error(body.detail ?? `ML HTTP ${res.status}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json();
}

export async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_URL}${path}`, {
    ...init,
    headers: { ...buildAuthHeaders(), ...(init?.headers as Record<string, string>) },
  });
  if (!res.ok) {
    const body = (await res.json().catch(() => ({}))) as EngineApiError;
    throw new Error(body.message ?? body.code ?? `HTTP ${res.status}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json();
}

export async function requestOptional<T>(path: string, init?: RequestInit): Promise<T | undefined> {
  const res = await fetch(`${API_URL}${path}`, {
    ...init,
    headers: { ...buildAuthHeaders(), ...(init?.headers as Record<string, string>) },
  });
  if (res.status === 404) {
    return undefined;
  }
  if (!res.ok) {
    const body = (await res.json().catch(() => ({}))) as EngineApiError;
    throw new Error(body.message ?? body.code ?? `HTTP ${res.status}`);
  }
  if (res.status === 204) return undefined;
  return res.json();
}

export async function requestBlob(
  path: string,
  init?: RequestInit,
): Promise<{ blob: Blob; filename?: string }> {
  const res = await fetch(`${API_URL}${path}`, {
    ...init,
    headers: { ...buildAuthHeaders(), ...(init?.headers as Record<string, string>) },
  });

  if (!res.ok) {
    const body = (await res.json().catch(() => ({}))) as EngineApiError;
    throw new Error(body.message ?? body.code ?? `HTTP ${res.status}`);
  }

  return {
    blob: await res.blob(),
    filename: parseContentDispositionFilename(res.headers.get('Content-Disposition')),
  };
}

export function json(method: string, body: unknown): RequestInit {
  return {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  };
}

function parseContentDispositionFilename(value: string | null): string | undefined {
  if (!value) return undefined;

  const match = /filename="([^"]+)"/i.exec(value) ?? /filename=([^;]+)/i.exec(value);
  return match?.[1]?.trim();
}

export function unsupported(feature: string): never {
  throw new Error(`${feature} is not supported by engine-api yet`);
}

export function unsupportedPromise<T>(feature: string): Promise<T> {
  return Promise.reject(new Error(`${feature} is not supported by engine-api yet`));
}

export function readStoredProfileId() {
  return readProfileId();
}

export function resolveProfileId(profileId?: string) {
  return resolveProfileSessionId(profileId);
}

export function writeStoredProfileId(profileId: string) {
  writeProfileId(profileId);
}

export function withProfileIdQuery(path: string) {
  return appendProfileIdQuery(path);
}
