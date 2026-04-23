from typing import Any

from app.enrichment.interview_prep.contract import InterviewPrepRequest


def build_interview_prep(context: InterviewPrepRequest) -> dict[str, Any]:
    analyzed_profile = context.analyzed_profile
    search_profile = context.search_profile
    job = context.ranked_job
    fit = context.deterministic_fit
    explanation = context.job_fit_explanation
    coaching = context.application_coach
    draft = context.cover_letter_draft
    feedback_state = context.feedback_state

    role_anchor = (
        analyzed_profile.primary_role.replace("_", " ")
        if analyzed_profile
        else search_profile.primary_role.replace("_", " ")
        if search_profile
        else "target role"
    )

    likely_topics: list[str] = []
    if fit.matched_roles:
        likely_topics.append(
            f"How your {', '.join(fit.matched_roles[:2])} background maps to this role."
        )
    if fit.matched_skills:
        likely_topics.append(
            f"Concrete discussion of {', '.join(fit.matched_skills[:3])} because those skills match deterministically."
        )
    if job.summary:
        likely_topics.append(f"Responsibilities and scope highlighted in the job summary: {job.summary}")
    if explanation and explanation.why_it_matches:
        likely_topics.extend(explanation.why_it_matches[:2])

    technical_focus: list[str] = []
    if fit.matched_skills:
        technical_focus.append(
            f"Prepare concrete examples for {', '.join(fit.matched_skills[:3])} using only experience already supported by the profile context."
        )
    if fit.matched_keywords:
        technical_focus.append(
            f"Be ready to explain overlap with {', '.join(fit.matched_keywords[:3])} in the job description."
        )
    if fit.reasons:
        technical_focus.append(f"Rehearse the deterministic evidence already called out: {fit.reasons[0]}")
    if coaching and coaching.interview_focus:
        technical_focus.extend(coaching.interview_focus[:2])

    behavioral_focus: list[str] = []
    behavioral_focus.append(
        f"Explain why this {role_anchor} direction is a grounded next step based on the current profile and search profile."
    )
    if feedback_state and feedback_state.current_job_feedback:
        if feedback_state.current_job_feedback.company_status == "whitelist":
            behavioral_focus.append(
                "Be ready to explain why this company is already on the positive target list."
            )
    if explanation and explanation.recommended_next_step:
        behavioral_focus.append(explanation.recommended_next_step)
    if draft and draft.tone_notes:
        behavioral_focus.extend(draft.tone_notes[:1])

    stories_to_prepare: list[str] = []
    if analyzed_profile and analyzed_profile.summary:
        stories_to_prepare.append(
            f"Choose one example that supports this profile summary without adding any facts beyond it: {analyzed_profile.summary}"
        )
    if fit.matched_skills:
        stories_to_prepare.append(
            f"Choose one example for {', '.join(fit.matched_skills[:2])} that is already supported by the available profile evidence."
        )
    if fit.reasons:
        stories_to_prepare.append(
            f"Prepare a concise story that backs up this deterministic reason: {fit.reasons[0]}"
        )
    if coaching and coaching.suggested_bullets:
        stories_to_prepare.extend(coaching.suggested_bullets[:2])

    questions_to_ask: list[str] = []
    questions_to_ask.append(
        f"What outcomes matter most for the {job.title} role in the first few months?"
    )
    if fit.matched_skills:
        questions_to_ask.append(
            f"Which parts of the role depend most on {', '.join(fit.matched_skills[:2])}?"
        )
    if explanation and explanation.missing_signals:
        questions_to_ask.append(
            f"Could you clarify this area that is not explicit in the current context: {explanation.missing_signals[0]}"
        )
    if job.work_mode_label or job.location_label:
        questions_to_ask.append(
            f"How does the team operate across {job.work_mode_label or 'the expected work mode'} and {job.location_label or 'the listed location constraints'}?"
        )

    risk_areas: list[str] = []
    if explanation and explanation.risks:
        risk_areas.extend(explanation.risks[:2])
    if explanation and explanation.missing_signals:
        risk_areas.extend(explanation.missing_signals[:2])
    if coaching and coaching.gaps_to_address:
        risk_areas.extend(coaching.gaps_to_address[:2])
    if draft and draft.evidence_gaps:
        risk_areas.extend(draft.evidence_gaps[:2])
    if not fit.matched_skills:
        risk_areas.append("The deterministic fit does not expose matched skills for this job yet.")
    if not analyzed_profile and not context.raw_profile_text:
        risk_areas.append("Profile evidence is limited, so interview examples may be too generic.")
    if fit.work_mode_match is False:
        risk_areas.append("Work-mode alignment is not clean in the deterministic context.")
    if fit.region_match is False:
        risk_areas.append("Region alignment is not clean in the deterministic context.")

    follow_up_plan: list[str] = []
    follow_up_plan.append(
        "Review the deterministic fit reasons and matched terms immediately before the interview."
    )
    if fit.matched_skills:
        follow_up_plan.append(
            f"Pick one verified example for each of {', '.join(fit.matched_skills[:2])} and keep the evidence concise."
        )
    if risk_areas:
        follow_up_plan.append(
            "Prepare short, candid responses for the listed risk areas instead of improvising unsupported details."
        )
    if questions_to_ask:
        follow_up_plan.append("Bring two or three grounded questions so the conversation stays evidence-based.")

    return {
        "prep_summary": (
            f"Prepare for {job.title} at {job.company_name} by centering the discussion on explicit {role_anchor} overlap, deterministic fit score {fit.score}, and examples that are already supported by the provided profile context."
        ),
        "likely_topics": likely_topics,
        "technical_focus": technical_focus,
        "behavioral_focus": behavioral_focus,
        "stories_to_prepare": stories_to_prepare,
        "questions_to_ask": questions_to_ask,
        "risk_areas": risk_areas,
        "follow_up_plan": follow_up_plan,
    }
