from typing import Any

from app.job_fit_explanation import JobFitExplanationPrompt, JobFitExplanationRequest


def build_job_fit_explanation(
    context: JobFitExplanationRequest, prompt: JobFitExplanationPrompt
) -> dict[str, Any]:
    analyzed_profile = context.analyzed_profile
    search_profile = context.search_profile
    job = context.ranked_job
    fit = context.deterministic_fit
    feedback_state = context.feedback_state

    fit_label = "good fit" if fit.score >= 70 else "risky fit" if fit.score < 40 else "mixed fit"
    role_anchor = (
        analyzed_profile.primary_role.replace("_", " ")
        if analyzed_profile
        else search_profile.primary_role.replace("_", " ")
        if search_profile
        else "current target role"
    )

    why_it_matches: list[str] = []
    if fit.matched_roles:
        why_it_matches.append(
            f"Deterministic role overlap exists with {', '.join(fit.matched_roles[:3])}."
        )
    if fit.matched_skills:
        why_it_matches.append(f"Matched skills include {', '.join(fit.matched_skills[:4])}.")
    if fit.reasons:
        why_it_matches.append(f"Deterministic evidence already highlights: {fit.reasons[0]}")

    risks: list[str] = []
    if fit.work_mode_match is False:
        risks.append("The job work mode conflicts with the current search profile.")
    if fit.region_match is False:
        risks.append("Region signals do not clearly align with the target search regions.")
    if feedback_state and feedback_state.current_job_feedback:
        if feedback_state.current_job_feedback.bad_fit:
            risks.append("This job is already marked as bad fit in feedback state.")
        if feedback_state.current_job_feedback.company_status == "blacklist":
            risks.append("The company is currently blacklisted, which makes the job operationally risky.")
    if fit.score < 40:
        risks.append("The deterministic fit score is low, so the match should be treated cautiously.")

    missing_signals: list[str] = []
    if not fit.matched_skills:
        missing_signals.append("The deterministic fit does not show direct matched skills yet.")
    if not fit.matched_keywords:
        missing_signals.append("Keyword overlap is limited, so narrative alignment is not strongly evidenced.")
    if not job.summary:
        missing_signals.append("The job presentation summary is missing, so the role narrative is less specific.")
    if not analyzed_profile:
        missing_signals.append("Analyzed profile context is limited, so explanation depth is constrained.")

    recommended_next_step = (
        f"Open the source posting and compare the first responsibilities against your {role_anchor} evidence before applying."
    )
    if fit.score >= 70 and fit.matched_skills:
        recommended_next_step = (
            f"Tailor the opening CV bullets to {', '.join(fit.matched_skills[:3])} and apply while the deterministic fit is strong."
        )

    application_angle_parts: list[str] = []
    if analyzed_profile and analyzed_profile.summary:
        application_angle_parts.append(analyzed_profile.summary)
    if fit.matched_skills:
        application_angle_parts.append(
            f"Lead with evidence around {', '.join(fit.matched_skills[:3])}."
        )
    elif fit.matched_roles:
        application_angle_parts.append(
            f"Frame the application around overlap with {', '.join(fit.matched_roles[:2])}."
        )

    return {
        "fit_summary": (
            f"This ranked job looks like a {fit_label} for {role_anchor} based on deterministic score {fit.score} and the available profile-job overlap."
        ),
        "why_it_matches": why_it_matches,
        "risks": risks,
        "missing_signals": missing_signals,
        "recommended_next_step": recommended_next_step,
        "application_angle": " ".join(application_angle_parts).strip(),
    }
