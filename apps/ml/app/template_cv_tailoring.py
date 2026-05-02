from __future__ import annotations

import re
from typing import Any

from app.enrichment.cv_tailoring.contract import CvTailoringRequest

COMMON_SKILL_ALIASES: dict[str, list[str]] = {
    "react": ["react", "react.js", "reactjs"],
    "react native": ["react native", "rn"],
    "typescript": ["typescript", "ts"],
    "javascript": ["javascript", "js"],
    "node.js": ["node.js", "nodejs", "node"],
    "next.js": ["next.js", "nextjs"],
    "python": ["python"],
    "rust": ["rust"],
    "postgresql": ["postgresql", "postgres", "psql"],
    "mongodb": ["mongodb", "mongo"],
    "redis": ["redis"],
    "docker": ["docker"],
    "kubernetes": ["kubernetes", "k8s"],
    "aws": ["aws", "amazon web services"],
    "gcp": ["gcp", "google cloud"],
    "azure": ["azure"],
    "graphql": ["graphql"],
    "rest api": ["rest api", "restful", "rest"],
    "ci/cd": ["ci/cd", "cicd", "github actions", "gitlab ci"],
    "swift": ["swift"],
    "kotlin": ["kotlin"],
    "java": ["java"],
    "c#": ["c#", ".net", "dotnet"],
    "angular": ["angular"],
    "vue": ["vue", "vue.js"],
    "tailwind": ["tailwind", "tailwindcss"],
    "testing": ["testing", "unit tests", "integration tests", "e2e"],
    "microservices": ["microservices", "micro-services"],
    "distributed systems": ["distributed systems", "distributed architecture"],
}


def _normalize_text(value: str | None) -> str:
    return re.sub(r"\s+", " ", value or "").strip()


def _normalize_skill(value: str) -> str:
    value = _normalize_text(value).lower()
    value = value.strip(" .,/;:|()[]{}")

    for canonical, aliases in COMMON_SKILL_ALIASES.items():
        if value == canonical or value in aliases:
            return canonical

    return value


def _title_case_skill(value: str) -> str:
    special = {
        "react": "React",
        "react native": "React Native",
        "typescript": "TypeScript",
        "javascript": "JavaScript",
        "node.js": "Node.js",
        "next.js": "Next.js",
        "postgresql": "Postgres",
        "mongodb": "MongoDB",
        "aws": "AWS",
        "gcp": "GCP",
        "rest api": "REST API",
        "ci/cd": "CI/CD",
        "c#": "C#",
    }

    return special.get(value, value.title())


def _dedupe(values: list[str], limit: int | None = None) -> list[str]:
    result: list[str] = []
    seen: set[str] = set()

    for value in values:
        normalized = _normalize_skill(value)

        if not normalized or normalized in seen:
            continue

        seen.add(normalized)
        result.append(_title_case_skill(normalized))

        if limit is not None and len(result) >= limit:
            break

    return result


def _contains_skill(text: str, skill: str) -> bool:
    normalized_text = text.lower()
    normalized_skill = _normalize_skill(skill)

    aliases = COMMON_SKILL_ALIASES.get(normalized_skill, [normalized_skill])

    return any(alias in normalized_text for alias in aliases)


def _infer_job_skills(context: CvTailoringRequest) -> list[str]:
    explicit_skills = [
        *context.job_required_skills,
        *context.job_nice_to_have_skills,
    ]

    if explicit_skills:
        return _dedupe(explicit_skills, limit=14)

    job_text = " ".join(
        [
            context.job_title or "",
            context.job_description or "",
        ]
    ).lower()

    inferred: list[str] = []

    for canonical, aliases in COMMON_SKILL_ALIASES.items():
        if any(alias in job_text for alias in aliases):
            inferred.append(canonical)

    return _dedupe(inferred, limit=14)


def _split_candidate_skills(context: CvTailoringRequest) -> list[str]:
    skills = list(context.candidate_skills)

    if not skills and context.candidate_cv_text:
        cv_text = context.candidate_cv_text.lower()

        for canonical, aliases in COMMON_SKILL_ALIASES.items():
            if any(alias in cv_text for alias in aliases):
                skills.append(canonical)

    return _dedupe(skills, limit=20)


def _is_bad_profile_summary(summary: str) -> bool:
    bad_markers = [
        "candidate is strongly aligned",
        "candidate is aligned",
        "signal quality",
        "alternative role directions",
        "based on signals",
        "applying to",
    ]

    normalized = summary.lower()

    return any(marker in normalized for marker in bad_markers)


def _build_summary(
    *,
    context: CvTailoringRequest,
    skills_to_highlight: list[str],
    skills_to_mention: list[str],
) -> str:
    title = _normalize_text(context.job_title) or "this role"
    profile_summary = _normalize_text(context.profile_summary)

    usable_profile_summary = (
        profile_summary
        if profile_summary and not _is_bad_profile_summary(profile_summary)
        else ""
    )

    strongest_skills = _dedupe(
        [
            *skills_to_highlight,
            *skills_to_mention,
        ],
        limit=6,
    )

    if usable_profile_summary:
        base = usable_profile_summary.rstrip(".")
    else:
        base = "Senior software engineer with experience building production web and product systems"

    if strongest_skills:
        skills_part = ", ".join(strongest_skills)
        return (
            f"{base}. Relevant for {title} through hands-on experience with "
            f"{skills_part}, product-focused delivery, and cross-functional engineering work."
        )

    return (
        f"{base}. Relevant for {title} through production engineering experience, "
        "ownership of technical delivery, and ability to adapt to role-specific requirements."
    )


def _gap_suggestion(skill: str) -> str:
    return (
        f"If you have real adjacent experience, mention it honestly near {skill}. "
        f"Do not claim direct {skill} experience unless it is true; instead connect it to related projects, tools, or responsibilities."
    )


def build_cv_tailoring(context: CvTailoringRequest) -> dict[str, Any]:
    candidate_skills = _split_candidate_skills(context)

    required_skills = _dedupe(context.job_required_skills, limit=10)
    nice_to_have_skills = _dedupe(context.job_nice_to_have_skills, limit=8)

    if not required_skills and not nice_to_have_skills:
        required_skills = _infer_job_skills(context)

    job_skills = _dedupe(
        [
            *required_skills,
            *nice_to_have_skills,
        ],
        limit=14,
    )

    candidate_normalized = {_normalize_skill(skill) for skill in candidate_skills}

    skills_to_highlight = [
        skill
        for skill in required_skills
        if _normalize_skill(skill) in candidate_normalized
    ]

    highlighted_normalized = {
        _normalize_skill(skill)
        for skill in skills_to_highlight
    }

    skills_to_mention = [
        skill
        for skill in nice_to_have_skills
        if _normalize_skill(skill) in candidate_normalized
        and _normalize_skill(skill) not in highlighted_normalized
    ]

    if not skills_to_mention:
        job_text = " ".join(
            [
                context.job_title or "",
                context.job_description or "",
            ]
        )

        skills_to_mention = [
            skill
            for skill in candidate_skills
            if _normalize_skill(skill) not in highlighted_normalized
            and (
                _contains_skill(job_text, skill)
                or _contains_skill(context.profile_summary or "", skill)
            )
        ][:5]

    job_normalized = [_normalize_skill(skill) for skill in job_skills]

    missing_skills = [
        _title_case_skill(skill)
        for skill in job_normalized
        if skill and skill not in candidate_normalized
    ]

    gaps_to_address = [
        {
            "skill": skill,
            "suggestion": _gap_suggestion(skill),
        }
        for skill in _dedupe(missing_skills, limit=5)
    ]

    skills_to_highlight = _dedupe(skills_to_highlight, limit=8)
    skills_to_mention = _dedupe(skills_to_mention, limit=8)

    summary_rewrite = _build_summary(
        context=context,
        skills_to_highlight=skills_to_highlight,
        skills_to_mention=skills_to_mention,
    )

    key_phrases = _dedupe(
        [
            *required_skills,
            *nice_to_have_skills,
        ],
        limit=12,
    )

    return {
        "skills_to_highlight": skills_to_highlight,
        "skills_to_mention": skills_to_mention,
        "gaps_to_address": gaps_to_address,
        "summary_rewrite": summary_rewrite,
        "key_phrases": key_phrases,
    }
