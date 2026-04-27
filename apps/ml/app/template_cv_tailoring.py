from typing import Any

from app.enrichment.cv_tailoring.contract import CvTailoringRequest


def build_cv_tailoring(context: CvTailoringRequest) -> dict[str, Any]:
    required_lower = {s.lower() for s in context.job_required_skills}
    nice_lower = {s.lower() for s in context.job_nice_to_have_skills}
    candidate_lower = {s.lower() for s in context.candidate_skills}

    skills_to_highlight = [
        s for s in context.candidate_skills if s.lower() in required_lower
    ]
    skills_to_mention = [
        s for s in context.candidate_skills
        if s.lower() in nice_lower and s.lower() not in required_lower
    ]

    gaps_to_address = []
    for skill in context.job_required_skills:
        if skill.lower() not in candidate_lower:
            gaps_to_address.append(
                {
                    "skill": skill,
                    "suggestion": f"Consider adding concrete evidence of {skill} to your CV.",
                }
            )

    skill_intro = ""
    if context.candidate_skills:
        top = context.candidate_skills[:3]
        skill_intro = f" with expertise in {', '.join(top)}"

    role = context.job_title or "the target role"
    summary_rewrite = f"Results-driven professional targeting {role}{skill_intro}."
    if context.profile_summary:
        summary_rewrite = (
            f"{context.profile_summary.rstrip('.')},"
            f" applying to {role}{skill_intro}."
        )

    key_phrases: list[str] = []
    seen: set[str] = set()
    for skill in list(context.job_required_skills) + list(context.job_nice_to_have_skills):
        key = skill.lower()
        if key not in seen:
            seen.add(key)
            key_phrases.append(skill)
        if len(key_phrases) >= 10:
            break

    return {
        "skills_to_highlight": skills_to_highlight,
        "skills_to_mention": skills_to_mention,
        "gaps_to_address": gaps_to_address[:10],
        "summary_rewrite": summary_rewrite,
        "key_phrases": key_phrases,
    }
