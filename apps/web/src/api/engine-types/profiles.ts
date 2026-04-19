export type EngineResume = {
  id: string;
  version: number;
  filename: string;
  raw_text: string;
  is_active: boolean;
  uploaded_at: string;
};

export type EngineMatchResult = {
  id: string;
  job_id: string;
  resume_id: string;
  score: number;
  matched_skills: string[];
  missing_skills: string[];
  notes: string;
  created_at: string;
};

export type EngineRoleCandidate = {
  role: string;
  score: number;
  confidence: number;
  matched_signals: string[];
};

export type EngineAnalyzeProfile = {
  summary: string;
  primary_role: string;
  seniority: string;
  skills: string[];
  keywords: string[];
  role_candidates?: EngineRoleCandidate[];
  suggested_search_terms?: string[];
};

export type EngineProfileAnalysis = {
  summary: string;
  primary_role: string;
  seniority: string;
  skills: string[];
  keywords: string[];
};

export type EngineProfile = {
  id: string;
  name: string;
  email: string;
  location?: string | null;
  raw_text: string;
  years_of_experience?: number | null;
  salary_min?: number | null;
  salary_max?: number | null;
  salary_currency?: string | null;
  languages?: string[] | null;
  analysis?: EngineProfileAnalysis | null;
  created_at: string;
  updated_at: string;
  skills_updated_at?: string | null;
};