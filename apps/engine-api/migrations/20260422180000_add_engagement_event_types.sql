-- Widen the event_type CHECK constraint to include engagement depth events.
ALTER TABLE user_events DROP CONSTRAINT IF EXISTS user_events_event_type_check;

ALTER TABLE user_events ADD CONSTRAINT user_events_event_type_check CHECK (
    event_type IN (
        'job_impression', 'job_opened', 'job_saved', 'job_unsaved',
        'job_hidden', 'job_unhidden', 'job_bad_fit', 'job_bad_fit_removed',
        'company_whitelisted', 'company_blacklisted', 'search_run',
        'fit_explanation_requested', 'application_coach_requested',
        'cover_letter_draft_requested', 'interview_prep_requested',
        'application_created',
        'job_scrolled_to_bottom', 'job_returned', 'job_shared'
    )
);
