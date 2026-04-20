import type {
  EngineRoleCatalogItemResponse,
  EngineRoleCatalogResponse as SharedEngineRoleCatalogResponse,
  EngineSourceId,
} from '@job-copilot/shared/search';

export type EngineSourceCatalogItem = {
  id: EngineSourceId;
  display_name: string;
};

export type EngineRoleCatalogItem = EngineRoleCatalogItemResponse;

export type EngineSourceCatalogResponse = {
  sources: EngineSourceCatalogItem[];
};

export type EngineRoleCatalogResponse = SharedEngineRoleCatalogResponse;
