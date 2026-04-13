# Entities

## User
Represents a user of the platform.

Fields:
- id
- email
- created_at
- updated_at

## SearchProfile
Represents search preferences.

Fields:
- id
- user_id
- keywords
- preferred_locations
- remote_preference
- seniority
- salary_expectation

## Job
Canonical job entity.

Fields:
- id
- title
- company_name
- location
- remote_type
- seniority
- description_text
- salary_min
- salary_max
- salary_currency
- posted_at
- last_seen_at
- is_active

## JobVariant
Source-specific copy of a job.

Fields:
- id
- job_id
- source
- source_job_id
- source_url
- raw_hash
- raw_payload
- fetched_at

## Application
Represents a user's job application state.

Fields:
- id
- user_id
- job_id
- status
- notes
- created_at
- updated_at

Statuses:
- saved
- applied
- interview
- rejected
- offer

## Contact
Represents a person related to an application.

Fields:
- id
- name
- email
- phone
- linkedin_url
- company
- role
- created_at

## Offer
Represents compensation and state after an application reaches offer stage.

Fields:
- id
- application_id
- status
- compensation_min
- compensation_max
- compensation_currency
- starts_at
- notes
- created_at
- updated_at

Offer statuses:
- draft
- received
- accepted
- declined
- expired

## Profile recency
Profile skill freshness is tracked separately from generic profile updates.

Fields:
- skills_updated_at
