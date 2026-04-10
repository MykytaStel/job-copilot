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