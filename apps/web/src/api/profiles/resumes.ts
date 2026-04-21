import type { ResumeUploadInput, ResumeVersion } from '@job-copilot/shared/profiles';
import { json, request } from '../client';
import type { EngineResume } from '../engine-types';
import { mapResume } from '../mappers';

export async function getResumes(): Promise<ResumeVersion[]> {
  const resumes = await request<EngineResume[]>('/api/v1/resumes');
  return resumes.map(mapResume);
}

export async function getActiveResume(): Promise<ResumeVersion> {
  const resume = await request<EngineResume>('/api/v1/resumes/active');
  return mapResume(resume);
}

export async function uploadResume(payload: ResumeUploadInput): Promise<ResumeVersion> {
  const resume = await request<EngineResume>(
    '/api/v1/resume/upload',
    json('POST', {
      filename: payload.filename,
      raw_text: payload.rawText,
    }),
  );
  return mapResume(resume);
}

export async function activateResume(id: string): Promise<ResumeVersion> {
  const resume = await request<EngineResume>(`/api/v1/resumes/${id}/activate`, json('POST', {}));
  return mapResume(resume);
}
