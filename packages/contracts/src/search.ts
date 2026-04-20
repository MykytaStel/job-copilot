import type { Contact } from './applications';
import type { JobPosting } from './jobs';
import type { EngineAnalyzeProfileResponse } from './profiles';

export interface SearchResults {
  jobs: JobPosting[];
  contacts: Contact[];
  page: number;
  perPage: number;
  hasMore: boolean;
}

export type EngineRoleId =
  | 'react_native_developer'
  | 'mobile_developer'
  | 'frontend_developer'
  | 'backend_developer'
  | 'fullstack_developer'
  | 'qa_engineer'
  | 'devops_engineer'
  | 'data_analyst'
  | 'ui_ux_designer'
  | 'product_manager'
  | 'project_manager'
  | 'marketing_specialist'
  | 'sales_manager'
  | 'customer_support_specialist'
  | 'recruiter'
  | 'generalist';

export type EngineTargetRegion =
  | 'ua'
  | 'eu'
  | 'eu_remote'
  | 'poland'
  | 'germany'
  | 'uk'
  | 'us';

export type EngineWorkMode = 'remote' | 'hybrid' | 'onsite';
export type EngineSourceId = 'djinni' | 'dou_ua' | 'work_ua' | 'robota_ua';

export interface EngineRoleCatalogItemResponse {
  id: EngineRoleId;
  display_name: string;
  deprecated_api_ids: string[];
  family?: string;
  is_fallback: boolean;
}

export interface EngineRoleCatalogResponse {
  roles: EngineRoleCatalogItemResponse[];
}

export interface EngineSearchPreferencesRequest {
  target_regions?: EngineTargetRegion[];
  work_modes?: EngineWorkMode[];
  preferred_roles?: string[];
  allowed_sources?: string[];
  include_keywords?: string[];
  exclude_keywords?: string[];
}

export interface EngineBuildSearchProfileRequest {
  raw_text: string;
  preferences?: EngineSearchPreferencesRequest;
}

export interface EngineSearchProfileResponse {
  primary_role: EngineRoleId;
  target_roles: EngineRoleId[];
  seniority: string;
  target_regions: EngineTargetRegion[];
  work_modes: EngineWorkMode[];
  allowed_sources: EngineSourceId[];
  search_terms: string[];
  exclude_terms: string[];
}

export interface EngineDeprecatedPreferredRoleReplacementResponse {
  received: string;
  normalized_to: EngineRoleId;
}

export interface EngineBuildSearchProfileWarningResponse {
  code: 'deprecated_preferred_roles';
  field: 'preferred_roles';
  message: string;
  replacements: EngineDeprecatedPreferredRoleReplacementResponse[];
}

export interface EngineBuildSearchProfileResponse {
  analyzed_profile: EngineAnalyzeProfileResponse;
  search_profile: EngineSearchProfileResponse;
  warnings?: EngineBuildSearchProfileWarningResponse[];
}

export interface EngineSearchProfileValidationErrorResponse {
  code: 'invalid_preferred_roles' | 'invalid_allowed_sources';
  field: 'preferred_roles' | 'allowed_sources';
  error: 'invalid_preferred_roles' | 'invalid_allowed_sources';
  message: string;
  invalid_values: string[];
  allowed_values: Array<EngineRoleId | EngineSourceId>;
}
