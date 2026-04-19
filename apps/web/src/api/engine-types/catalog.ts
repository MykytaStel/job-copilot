export type EngineSourceCatalogItem = {
  id: string;
  display_name: string;
};

export type EngineRoleCatalogItem = {
  id: string;
  display_name: string;
  deprecated_api_ids: string[];
  family?: string;
  is_fallback: boolean;
};

export type EngineSourceCatalogResponse = {
  sources: EngineSourceCatalogItem[];
};

export type EngineRoleCatalogResponse = {
  roles: EngineRoleCatalogItem[];
};