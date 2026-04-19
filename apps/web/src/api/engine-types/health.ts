export type EngineApiError = {
  code?: string;
  message?: string;
  details?: unknown;
};

export type EngineHealthResponse = {
  status: string;
  database: {
    status: string;
    configured: boolean;
    migrations_enabled_on_startup: boolean;
  };
};
