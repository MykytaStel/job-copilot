import type { EngineRoleId } from './search';

export interface CandidateProfile {
  id: string;
  name: string;
  email: string;
  location?: string;
  yearsOfExperience?: number;
  salaryMin?: number;
  salaryMax?: number;
  salaryCurrency: string;
  languages: string[];
  summary?: string;
  skills: string[];
  updatedAt: string;
  skillsUpdatedAt?: string;
}

export interface CandidateProfileInput {
  name: string;
  email: string;
  location?: string;
  yearsOfExperience?: number;
  salaryMin?: number;
  salaryMax?: number;
  salaryCurrency?: string;
  languages: string[];
  summary?: string;
  skills: string[];
}

export interface ResumeVersion {
  id: string;
  version: number;
  filename: string;
  rawText: string;
  isActive: boolean;
  uploadedAt: string;
}

export interface ResumeUploadInput {
  filename: string;
  rawText: string;
}

/** @deprecated use ResumeVersion */
export type Resume = ResumeVersion;

export interface EngineAnalyzeProfileRequest {
  raw_text: string;
}

export interface EngineRoleCandidateResponse {
  role: EngineRoleId;
  score: number;
  confidence: number;
  matched_signals: string[];
}

export interface EngineAnalyzeProfileResponse {
  summary: string;
  primary_role: EngineRoleId;
  seniority: string;
  skills: string[];
  keywords: string[];
  role_candidates: EngineRoleCandidateResponse[];
  suggested_search_terms: string[];
}
