from app.api import app
from app.scoring import normalize_text, score_job, tokenize
from app.service_dependencies import (
    get_application_coach_service,
    get_cover_letter_draft_service,
    get_interview_prep_service,
    get_job_fit_explanation_service,
    get_profile_insights_service,
    get_weekly_guidance_service,
)
