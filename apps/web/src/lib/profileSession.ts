const PROFILE_ID_STORAGE_KEY = 'engine_api_profile_id';

function canUseStorage() {
  return typeof window !== 'undefined' && !!window.localStorage;
}

export function readProfileId() {
  if (!canUseStorage()) {
    return null;
  }

  return window.localStorage.getItem(PROFILE_ID_STORAGE_KEY);
}

export function resolveProfileId(profileId?: string) {
  return profileId ?? readProfileId() ?? undefined;
}

export function writeProfileId(profileId: string) {
  if (!canUseStorage()) {
    return;
  }

  window.localStorage.setItem(PROFILE_ID_STORAGE_KEY, profileId);
}

export function withProfileIdQuery(path: string) {
  const profileId = readProfileId();
  if (!profileId) {
    return path;
  }

  const separator = path.includes('?') ? '&' : '?';
  return `${path}${separator}profile_id=${encodeURIComponent(profileId)}`;
}
