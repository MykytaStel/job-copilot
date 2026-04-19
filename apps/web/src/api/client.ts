import type { EngineApiError } from './engine-types/health';

const API_URL = import.meta.env.VITE_ENGINE_API_URL?.trim() || 'http://localhost:8080';
const ML_URL = import.meta.env.VITE_ML_URL?.trim() || 'http://localhost:8000';
const PROFILE_ID_KEY = 'engine_api_profile_id';

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
  return res.json();
}

export async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_URL}${path}`, init);
  if (!res.ok) {
    const body = (await res.json().catch(() => ({}))) as EngineApiError;
    throw new Error(body.message ?? body.code ?? `HTTP ${res.status}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json();
}

export async function requestOptional<T>(path: string, init?: RequestInit): Promise<T | undefined> {
  const res = await fetch(`${API_URL}${path}`, init);
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

export function json(method: string, body: unknown): RequestInit {
  return {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  };
}

export function unsupported(feature: string): never {
  throw new Error(`${feature} is not supported by engine-api yet`);
}

export function unsupportedPromise<T>(feature: string): Promise<T> {
  return Promise.reject(new Error(`${feature} is not supported by engine-api yet`));
}

export function readStoredProfileId() {
  return window.localStorage.getItem(PROFILE_ID_KEY);
}

export function resolveProfileId(profileId?: string) {
  return profileId ?? readStoredProfileId() ?? undefined;
}

export function writeStoredProfileId(profileId: string) {
  window.localStorage.setItem(PROFILE_ID_KEY, profileId);
}

export function withProfileIdQuery(path: string) {
  const profileId = readStoredProfileId();
  if (!profileId) return path;

  const separator = path.includes('?') ? '&' : '?';
  return `${path}${separator}profile_id=${encodeURIComponent(profileId)}`;
}
