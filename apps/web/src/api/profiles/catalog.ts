import { request } from '../client';
import type { EngineRoleCatalogResponse, EngineSourceCatalogResponse } from '../engine-types';
import type { RoleCatalogItem, SourceCatalogItem } from './types';

export async function getSources(): Promise<SourceCatalogItem[]> {
  const response = await request<EngineSourceCatalogResponse>('/api/v1/sources');
  return response.sources.map((source) => ({
    id: source.id,
    displayName: source.display_name,
  }));
}

export async function getRoles(): Promise<RoleCatalogItem[]> {
  const response = await request<EngineRoleCatalogResponse>('/api/v1/roles');
  return response.roles.map((role) => ({
    id: role.id,
    displayName: role.display_name,
    family: role.family,
    isFallback: role.is_fallback,
  }));
}
