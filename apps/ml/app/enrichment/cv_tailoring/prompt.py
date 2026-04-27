import json

from app.enrichment.cv_tailoring.contract import (
    CvTailoringPrompt,
    CvTailoringRequest,
    cv_tailoring_suggestions_schema,
)


def build_cv_tailoring_prompt(context: CvTailoringRequest) -> CvTailoringPrompt:
    prompt_context = {
        "profile_id": context.profile_id,
        "job_id": context.job_id,
        "profile_summary": context.profile_summary,
        "candidate_skills": context.candidate_skills,
        "job_title": context.job_title,
        "job_required_skills": context.job_required_skills,
        "job_nice_to_have_skills": context.job_nice_to_have_skills,
    }
    if context.job_description:
        prompt_context["job_description"] = context.job_description
    if context.candidate_cv_text:
        prompt_context["candidate_cv_text"] = context.candidate_cv_text

    output_schema = cv_tailoring_suggestions_schema()

    return CvTailoringPrompt(
        system_instructions=(
            "You are a CV tailoring advisor for a job search copilot. "
            "Given the candidate profile and a target job, identify which skills to highlight, "
            "which additional skills to mention, any skill gaps to address with actionable suggestions, "
            "a rewritten profile summary optimized for the role, and key phrases from the job. "
            "Use only evidence present in the provided context. Do not invent skills, experience, or achievements. "
            "Keep suggestions specific and actionable. "
            "Return plain JSON only. Do not use markdown, bullets, headings, or code fences."
        ),
        context_payload=json.dumps(prompt_context, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema_expectations=json.dumps(output_schema, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema=output_schema,
    )
