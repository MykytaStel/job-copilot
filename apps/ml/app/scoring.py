from __future__ import annotations

from app.engine_api_client import EngineJobLifecycle, EngineProfile
from app.scoring_evidence import build_evidence
from app.scoring_sections import SkillSections, parse_skill_sections, section_weight_for_term
from app.scoring_terms import (
    indexed_terms,
    job_terms,
    overlap,
    profile_terms,
    unique_preserving_order,
)
from app.text_normalization import tokenize

__all__ = [
    "SkillSections",
    "build_evidence",
    "indexed_terms",
    "job_terms",
    "overlap",
    "parse_skill_sections",
    "profile_terms",
    "score_job",
    "section_weight_for_term",
    "unique_preserving_order",
]


def score_job(
    profile: EngineProfile, job: EngineJobLifecycle
) -> tuple[int, list[str], list[str], list[str]]:
    profile_values = profile_terms(profile)
    job_values = job_terms(job)
    profile_index = indexed_terms(profile_values)
    job_index = indexed_terms(job_values)

    if not profile_index:
        return 0, [], [], ["profile has no usable terms yet"]

    sections = parse_skill_sections(job.description_text)
    job_keys = {key for key, _, _ in job_index}
    title_keys = {key for key, _, _ in indexed_terms(tokenize(job.title))}

    weighted_profile_index = [
        (key, display, base_weight * section_weight_for_term(key, sections))
        for key, display, base_weight in profile_index
    ]

    matched_terms = [display for key, display, _ in weighted_profile_index if key in job_keys]
    title_matches = [display for key, display, _ in weighted_profile_index if key in title_keys]

    total_weight = sum(weight for _, _, weight in weighted_profile_index)
    matched_weight = sum(weight for key, _, weight in weighted_profile_index if key in job_keys)
    title_weight = sum(weight for key, _, weight in weighted_profile_index if key in title_keys)

    overlap_ratio = matched_weight / total_weight if total_weight else 0.0
    title_bonus = min(int(round(title_weight * 8)), 24)

    seniority_bonus = 0

    if profile.analysis and profile.analysis.seniority and job.seniority:
        if profile.analysis.seniority.lower() == job.seniority.lower():
            seniority_bonus = 10

    active_bonus = 5 if job.is_active else 0
    lifecycle_bonus = 3 if job.lifecycle_stage == "reactivated" else 0

    score = min(
        int(round(overlap_ratio * 70))
        + title_bonus
        + seniority_bonus
        + active_bonus
        + lifecycle_bonus,
        100,
    )

    missing_terms = [
        display
        for key, display, _ in sorted(
            weighted_profile_index,
            key=lambda item: item[2],
            reverse=True,
        )
        if key not in job_keys
    ][:8]

    evidence = build_evidence(profile, job, matched_terms, title_matches)

    return score, matched_terms[:10], missing_terms, evidence
