from __future__ import annotations

from app.engine_api_client import EngineJobLifecycle, EngineProfile
from app.text_normalization import normalize_term_for_output, normalize_text, tokenize

SENIORITY_TERMS = {
    "intern",
    "junior",
    "middle",
    "mid",
    "senior",
    "lead",
    "staff",
    "principal",
    "head",
    "director",
}

CONTACT_NOISE_TERMS = {
    "email",
    "mail",
    "phone",
    "telegram",
    "linkedin",
    "contact",
    "contacts",
}


def unique_preserving_order(values: list[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []

    for value in values:
        if value in seen:
            continue

        seen.add(value)
        result.append(value)

    return result


def term_key(value: str) -> str:
    return normalize_text(value)


def term_weight(value: str) -> float:
    normalized = term_key(value)

    if not normalized:
        return 0.0

    if "_" in normalized:
        return 1.35

    return 1.0


def indexed_terms(values: list[str]) -> list[tuple[str, str, float]]:
    indexed: list[tuple[str, str, float]] = []
    seen: set[str] = set()

    for value in values:
        key = term_key(value)

        if not key or key in SENIORITY_TERMS or key in seen:
            continue

        seen.add(key)
        indexed.append((key, normalize_term_for_output(value), term_weight(value)))

    return indexed


def profile_terms(profile: EngineProfile) -> list[str]:
    analysis = profile.analysis

    if analysis:
        terms = tokenize(analysis.primary_role)
        _append_analysis_terms(terms, analysis.skills)
        _append_analysis_terms(terms, analysis.keywords)
    else:
        identity_terms = _profile_identity_terms(profile)
        terms = [
            term
            for term in tokenize(profile.raw_text)
            if not _is_profile_noise_term(term, identity_terms)
        ]

    return unique_preserving_order(terms)[:40]


def job_terms(job: EngineJobLifecycle) -> list[str]:
    return unique_preserving_order(
        tokenize(
            job.title,
            job.company_name,
            job.location,
            job.remote_type,
            job.seniority,
            job.description_text,
            job.presentation.summary,
            job.presentation.location_label,
            job.presentation.work_mode_label,
        )
    )


def overlap(profile_values: list[str], job_values: list[str]) -> list[str]:
    job_keys = {key for key, _, _ in indexed_terms(job_values)}

    return [display for key, display, _ in indexed_terms(profile_values) if key in job_keys]


def _profile_identity_terms(profile: EngineProfile) -> set[str]:
    return set(tokenize(profile.name, profile.email, profile.location))


def _is_profile_noise_term(term: str, identity_terms: set[str]) -> bool:
    normalized = term_key(term)

    if not normalized:
        return True

    if normalized in identity_terms or normalized in CONTACT_NOISE_TERMS:
        return True

    return normalized.isdigit() and len(normalized) >= 7


def _append_analysis_terms(terms: list[str], values: list[str]) -> None:
    for value in values:
        normalized = normalize_term_for_output(value)

        if normalized:
            terms.append(normalized)
