import { describe, expect, it } from 'vitest';

import { mapApplicationDetail, mapOffer } from '../src/api/mappers';
import type { EngineApplicationDetail, EngineOffer } from '../src/api/engine-types';

describe('api mappers', () => {
  it('maps offer without enum casts', () => {
    const offer: EngineOffer = {
      id: 'offer-1',
      application_id: 'app-1',
      status: 'extended',
      compensation_min: 1000,
      compensation_max: 2000,
      compensation_currency: 'USD',
      starts_at: '2026-04-20T10:00:00Z',
      notes: 'Test offer',
      created_at: '2026-04-19T10:00:00Z',
      updated_at: '2026-04-19T11:00:00Z',
    };

    expect(mapOffer(offer)).toEqual({
      id: 'offer-1',
      applicationId: 'app-1',
      status: 'extended',
      compensationMin: 1000,
      compensationMax: 2000,
      compensationCurrency: 'USD',
      startsAt: '2026-04-20T10:00:00Z',
      notes: 'Test offer',
      createdAt: '2026-04-19T10:00:00Z',
      updatedAt: '2026-04-19T11:00:00Z',
    });
  });

  it('maps application detail contact relationship and activity type directly', () => {
    const detail: EngineApplicationDetail = {
      id: 'app-1',
      job_id: 'job-1',
      resume_id: 'resume-1',
      status: 'saved',
      applied_at: null,
      due_date: null,
      updated_at: '2026-04-19T12:00:00Z',
      job: {
        id: 'job-1',
        title: 'Frontend Engineer',
        company_name: 'Acme',
        description_text: 'React TypeScript',
        location: 'Kyiv',
        remote_type: 'remote',
        seniority: 'middle',
        salary_min: null,
        salary_max: null,
        salary_currency: null,
        posted_at: null,
        first_seen_at: '2026-04-18T10:00:00Z',
        last_seen_at: '2026-04-19T10:00:00Z',
        is_active: true,
        inactivated_at: null,
        reactivated_at: null,
        lifecycle_stage: 'active',
        primary_variant: null,
        presentation: {
          title: 'Frontend Engineer',
          company: 'Acme',
          summary: null,
          summary_quality: null,
          summary_fallback: false,
          description_quality: 'good',
          location_label: 'Kyiv',
          work_mode_label: 'Remote',
          source_label: 'Manual',
          outbound_url: null,
          salary_label: null,
          freshness_label: null,
          badges: [],
        },
        feedback: {
          saved: false,
          hidden: false,
          bad_fit: false,
          company_status: null,
        },
      },
      resume: {
        id: 'resume-1',
        version: 1,
        filename: 'resume.pdf',
        raw_text: 'resume text',
        is_active: true,
        uploaded_at: '2026-04-18T09:00:00Z',
      },
      offer: null,
      notes: [],
      contacts: [
        {
          id: 'link-1',
          application_id: 'app-1',
          relationship: 'recruiter',
          contact: {
            id: 'contact-1',
            name: 'Jane Doe',
            email: 'jane@example.com',
            phone: null,
            linkedin_url: null,
            company: 'Acme',
            role: 'Recruiter',
            created_at: '2026-04-19T08:00:00Z',
          },
        },
      ],
      activities: [
        {
          id: 'act-1',
          application_id: 'app-1',
          activity_type: 'note',
          description: 'Reached out',
          happened_at: '2026-04-19T09:00:00Z',
          created_at: '2026-04-19T09:00:00Z',
        },
      ],
      tasks: [],
    };

    const mapped = mapApplicationDetail(detail);

    expect(mapped.contacts[0]?.relationship).toBe('recruiter');
    expect(mapped.activities[0]?.type).toBe('note');
  });
});
