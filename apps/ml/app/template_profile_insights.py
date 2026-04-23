from typing import Any

from app.enrichment.profile_insights.contract import LlmContextRequest, ProfileInsightsPrompt


def build_profile_insights(
    context: LlmContextRequest, prompt: ProfileInsightsPrompt
) -> dict[str, Any]:
    analyzed_profile = context.analyzed_profile
    role = analyzed_profile.primary_role.replace("_", " ") if analyzed_profile else "current target role"
    seniority = analyzed_profile.seniority if analyzed_profile and analyzed_profile.seniority else ""
    skills = analyzed_profile.skills if analyzed_profile else context.profile_skills
    keywords = analyzed_profile.keywords if analyzed_profile else context.profile_keywords

    strengths = []
    if analyzed_profile and analyzed_profile.summary:
        if seniority:
            strengths.append(f"Clear profile positioning around {role} at {seniority} level.")
        else:
            strengths.append(f"Clear profile positioning around {role}.")
    if skills:
        strengths.append(f"Relevant skills are already explicit: {', '.join(skills[:3])}.")
    if context.feedback_summary.saved_jobs_count > 0:
        strengths.append("Saved jobs indicate the current direction already has positive traction.")

    risks = []
    if not skills:
        risks.append("The profile context does not expose enough concrete skills yet.")
    if context.feedback_summary.bad_fit_jobs_count > 0:
        risks.append("Bad-fit feedback suggests the current search may still be too broad.")
    if context.feedback_summary.blacklisted_companies_count > 0:
        risks.append("Company blacklist signals narrow the practical target set.")

    focus_areas = []
    focus_areas.extend(skills[:3])
    focus_areas.extend(keyword for keyword in keywords[:3] if keyword not in focus_areas)

    search_terms = []
    if analyzed_profile:
        search_terms.append(role)
        search_terms.extend(skill for skill in skills[:3] if skill.lower() not in {term.lower() for term in search_terms})
        search_terms.extend(
            keyword
            for keyword in keywords[:3]
            if keyword.lower() not in {term.lower() for term in search_terms}
        )

    return {
        "profile_summary": (
            analyzed_profile.summary
            if analyzed_profile
            else "The profile needs more deterministic analysis before enrichment can be specific."
        ),
        "search_strategy_summary": (
            f"Anchor the search around {role} roles, keep filters aligned with the current deterministic profile, "
            "and use feedback to narrow terms instead of expanding into unrelated directions."
        ),
        "strengths": strengths,
        "risks": risks,
        "recommended_actions": [
            "Prioritize applications where the title and first responsibilities align with the current target role.",
            "Use saved and bad-fit feedback to tighten search terms before expanding volume.",
        ],
        "top_focus_areas": focus_areas,
        "search_term_suggestions": search_terms,
        "application_strategy": [
            "Apply first to roles that match the primary role and strongest listed skills.",
            "Skip listings that conflict with repeated bad-fit or blacklist feedback signals.",
        ],
    }
