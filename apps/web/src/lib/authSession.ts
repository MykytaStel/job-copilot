const TOKEN_KEY = 'engine_api_jwt_token';

function canUseStorage() {
  return (
    typeof window !== 'undefined' &&
    !!window.localStorage &&
    typeof window.localStorage.getItem === 'function' &&
    typeof window.localStorage.setItem === 'function' &&
    typeof window.localStorage.removeItem === 'function'
  );
}

export function readToken(): string | null {
  if (!canUseStorage()) return null;
  return window.localStorage.getItem(TOKEN_KEY);
}

export function writeToken(token: string) {
  if (!canUseStorage()) return;
  window.localStorage.setItem(TOKEN_KEY, token);
}

export function clearToken() {
  if (!canUseStorage()) return;
  window.localStorage.removeItem(TOKEN_KEY);
}

export function hasToken(): boolean {
  return !!readToken();
}

export function buildAuthHeaders(): Record<string, string> {
  const token = readToken();
  if (!token) return {};
  return { Authorization: `Bearer ${token}` };
}
