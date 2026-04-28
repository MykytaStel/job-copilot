import { request } from './client';

export type UserDataExport = Record<string, unknown>;

export async function exportUserData(): Promise<UserDataExport> {
  return request<UserDataExport>('/api/v1/export');
}
