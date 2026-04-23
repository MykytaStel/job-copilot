from typing import Any

from app.enrichment.cover_letter_draft.contract import CoverLetterDraftRequest


def build_cover_letter_draft(context: CoverLetterDraftRequest) -> dict[str, Any]:
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
