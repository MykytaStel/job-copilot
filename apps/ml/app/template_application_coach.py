from typing import Any

from app.enrichment.application_coach.contract import ApplicationCoachRequest


def build_application_coach(context: ApplicationCoachRequest) -> dict[str, Any]:
    analyzed_profile = context.analyzed_profile
    search_profile = context.search_profile
    job = context.ranked_job
    fit = context.deterministic_fit
    explanation = context.job_fit_explanation
    feedback_state = context.feedback_state

    role_anchor = (
        analyzed_profile.primary_role.replace("_", " ")
        if analyzed_profile
        else search_profile.primary_role.replace("_", " ")
        if search_profile
        else "current target role"
    )

    resume_focus_points: list[str] = []
    if fit.matched_roles:
        resume_focus_points.append(
            f"Keep the resume anchored to {', '.join(fit.matched_roles[:2])} language."
        )
    if fit.matched_skills:
        resume_focus_points.append(
            f"Move {', '.join(fit.matched_skills[:3])} closer to the top of the resume."
        )
    if explanation and explanation.application_angle:
        resume_focus_points.append(explanation.application_angle)

    suggested_bullets: list[str] = []
    if analyzed_profile and analyzed_profile.summary:
        suggested_bullets.append(
            f"Highlight experience already reflected in the profile summary: {analyzed_profile.summary}"
        )
    if fit.matched_skills:
        suggested_bullets.append(
            f"Reframe existing work around {', '.join(fit.matched_skills[:3])} because those skills match this job."
        )
    if fit.reasons:
        suggested_bullets.append(
            f"Use deterministic fit evidence as framing for a bullet: {fit.reasons[0]}"
        )

    cover_letter_angles: list[str] = []
    if explanation and explanation.fit_summary:
        cover_letter_angles.append(explanation.fit_summary)
    if job.summary:
        cover_letter_angles.append(
            f"Connect existing profile evidence to the job summary: {job.summary}"
        )
    if feedback_state and feedback_state.current_job_feedback:
        if feedback_state.current_job_feedback.company_status == "whitelist":
            cover_letter_angles.append(
                "Mention why this company already fits the current target list."
            )

    interview_focus: list[str] = []
    if fit.matched_skills:
        interview_focus.append(
            f"Prepare concrete examples around {', '.join(fit.matched_skills[:3])}."
        )
    if fit.matched_keywords:
        interview_focus.append(
            f"Be ready to explain overlap with {', '.join(fit.matched_keywords[:3])}."
        )
    if explanation and explanation.missing_signals:
        interview_focus.extend(explanation.missing_signals[:2])

    gaps_to_address: list[str] = []
    if not fit.matched_skills:
        gaps_to_address.append("The deterministic fit does not expose matched skills for this job yet.")
    if fit.work_mode_match is False:
        gaps_to_address.append("The current search work-mode preference does not align cleanly.")
    if fit.region_match is False:
        gaps_to_address.append("The current target regions do not align cleanly with this job.")
    if not analyzed_profile and not context.raw_profile_text:
        gaps_to_address.append("Profile evidence is limited, so tailoring depth is constrained.")

    red_flags: list[str] = []
    if fit.score < 40:
        red_flags.append("Low deterministic fit score means the application should be treated cautiously.")
    if feedback_state and feedback_state.current_job_feedback:
        if feedback_state.current_job_feedback.bad_fit:
            red_flags.append("This job is already marked as bad fit in feedback.")
        if feedback_state.current_job_feedback.company_status == "blacklist":
            red_flags.append("The company is blacklisted in current feedback state.")

    return {
        "application_summary": (
            f"Tailor this application around the existing {role_anchor} evidence that already overlaps with {job.title} at {job.company_name}, while keeping claims bounded to deterministic score {fit.score} and the provided profile context."
        ),
        "resume_focus_points": resume_focus_points,
        "suggested_bullets": suggested_bullets,
        "cover_letter_angles": cover_letter_angles,
        "interview_focus": interview_focus,
        "gaps_to_address": gaps_to_address,
        "red_flags": red_flags,
    }
