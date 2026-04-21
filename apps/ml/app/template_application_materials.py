from typing import Any

from app.application_coach import ApplicationCoachPrompt, ApplicationCoachRequest
from app.cover_letter_draft import CoverLetterDraftPrompt, CoverLetterDraftRequest
from app.interview_prep import InterviewPrepPrompt, InterviewPrepRequest


def build_application_coach(
    context: ApplicationCoachRequest, prompt: ApplicationCoachPrompt
) -> dict[str, Any]:
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


def build_cover_letter_draft(
    context: CoverLetterDraftRequest, prompt: CoverLetterDraftPrompt
) -> dict[str, Any]:
    analyzed_profile = context.analyzed_profile
    search_profile = context.search_profile
    job = context.ranked_job
    fit = context.deterministic_fit
    explanation = context.job_fit_explanation
    coaching = context.application_coach
    feedback_state = context.feedback_state

    role_anchor = (
        analyzed_profile.primary_role.replace("_", " ")
        if analyzed_profile
        else search_profile.primary_role.replace("_", " ")
        if search_profile
        else "target role"
    )

    key_claims_used: list[str] = []
    if analyzed_profile and analyzed_profile.summary:
        key_claims_used.append(analyzed_profile.summary)
    if fit.matched_roles:
        key_claims_used.append(f"Role overlap is explicit with {', '.join(fit.matched_roles[:2])}.")
    if fit.matched_skills:
        key_claims_used.append(f"Matched skills include {', '.join(fit.matched_skills[:3])}.")
    if fit.reasons:
        key_claims_used.append(f"Deterministic fit evidence: {fit.reasons[0]}")
    if explanation and explanation.application_angle:
        key_claims_used.append(explanation.application_angle)

    evidence_gaps: list[str] = []
    if explanation and explanation.missing_signals:
        evidence_gaps.extend(explanation.missing_signals[:2])
    if coaching and coaching.gaps_to_address:
        evidence_gaps.extend(coaching.gaps_to_address[:2])
    if not analyzed_profile and not context.raw_profile_text:
        evidence_gaps.append("Profile evidence is limited, so personalization depth is constrained.")
    if not fit.matched_skills:
        evidence_gaps.append("The deterministic fit does not expose matched skills for this job yet.")
    if fit.region_match is False:
        evidence_gaps.append("Region alignment is not explicit in the deterministic context.")
    if fit.work_mode_match is False:
        evidence_gaps.append("Work-mode alignment is not explicit in the deterministic context.")

    opening_parts: list[str] = [f"I am applying for the {job.title} role at {job.company_name}."]
    if analyzed_profile and analyzed_profile.summary:
        opening_parts.append(
            f"The provided profile context positions me as {analyzed_profile.summary}"
        )
    elif fit.matched_roles:
        opening_parts.append(
            f"The deterministic fit shows direct overlap with {', '.join(fit.matched_roles[:2])} work."
        )
    if fit.matched_skills:
        opening_parts.append(
            f"The strongest explicit overlap in the available evidence is around {', '.join(fit.matched_skills[:3])}."
        )

    body_paragraphs: list[str] = []
    if fit.reasons or explanation and explanation.fit_summary:
        evidence_parts: list[str] = []
        if fit.reasons:
            evidence_parts.append(fit.reasons[0])
        if explanation and explanation.fit_summary:
            evidence_parts.append(explanation.fit_summary)
        body_paragraphs.append(
            " ".join(
                [
                    "The deterministic context points to a grounded match for this role.",
                    *evidence_parts,
                ]
            ).strip()
        )

    second_body_parts: list[str] = []
    if job.summary:
        second_body_parts.append(f"The job summary emphasizes {job.summary}")
    if explanation and explanation.application_angle:
        second_body_parts.append(explanation.application_angle)
    elif coaching and coaching.cover_letter_angles:
        second_body_parts.append(coaching.cover_letter_angles[0])
    elif fit.matched_keywords:
        second_body_parts.append(
            f"The available overlap also includes {', '.join(fit.matched_keywords[:3])}."
        )
    if search_profile and search_profile.target_roles:
        second_body_parts.append(
            f"This role also aligns with the active search direction toward {', '.join(search_profile.target_roles[:2])}."
        )
    if second_body_parts:
        body_paragraphs.append(" ".join(second_body_parts).strip())

    if feedback_state and feedback_state.current_job_feedback:
        if feedback_state.current_job_feedback.company_status == "whitelist":
            body_paragraphs.append(
                "The current feedback state also shows this company is already on the positive target list."
            )
        elif feedback_state.current_job_feedback.company_status == "blacklist":
            evidence_gaps.append(
                "The company is blacklisted in feedback state, so the draft should be used cautiously."
            )

    tone_notes: list[str] = ["Keep the tone direct, tailored, and evidence-based."]
    if evidence_gaps:
        tone_notes.append("Acknowledge alignment without overstating unsupported depth.")
    if fit.score >= 70:
        tone_notes.append("Confident language is reasonable because deterministic fit is strong.")
    else:
        tone_notes.append("Keep the draft measured because the deterministic fit is mixed.")

    closing_parts = [
        f"I would welcome the opportunity to discuss how the available {role_anchor} evidence could support the needs of {job.company_name}."
    ]
    if evidence_gaps:
        closing_parts.append(
            "Any deeper claims should stay limited to examples that can be verified from the existing profile context."
        )

    return {
        "draft_summary": (
            f"Frame the letter around explicit {role_anchor} overlap, the deterministic fit score of {fit.score}, and the matched evidence already present in the profile and job context."
        ),
        "opening_paragraph": " ".join(opening_parts).strip(),
        "body_paragraphs": body_paragraphs,
        "closing_paragraph": " ".join(closing_parts).strip(),
        "key_claims_used": key_claims_used,
        "evidence_gaps": evidence_gaps,
        "tone_notes": tone_notes,
    }


def build_interview_prep(
    context: InterviewPrepRequest, prompt: InterviewPrepPrompt
) -> dict[str, Any]:
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
