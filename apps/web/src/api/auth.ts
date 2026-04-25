import { request } from './client';

export interface AuthResponse {
  token: string;
  profile_id: string;
  expires_at: string;
}

export async function register(params: {
  name: string;
  email: string;
  raw_text: string;
}): Promise<AuthResponse> {
  return request<AuthResponse>('/api/v1/auth/register', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(params),
  });
}

export async function login(params: { email: string }): Promise<AuthResponse> {
  return request<AuthResponse>('/api/v1/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(params),
  });
}
