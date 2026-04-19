export type EngineContact = {
  id: string;
  name: string;
  email?: string | null;
  phone?: string | null;
  linkedin_url?: string | null;
  company?: string | null;
  role?: string | null;
  created_at: string;
};

export type EngineContactsResponse = {
  contacts: EngineContact[];
};
