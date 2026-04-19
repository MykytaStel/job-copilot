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


def build_evidence(
    profile: EngineProfile,
    job: EngineJobLifecycle,
    matched_terms: list[str],
    title_matches: list[str],
) -> list[str]:
    evidence: list[str] = []

    if title_matches:
        evidence.append(f"title overlap: {', '.join(title_matches[:5])}")
    if matched_terms:
        evidence.append(f"shared terms: {', '.join(matched_terms[:8])}")

    seniority = profile.analysis.seniority if profile.analysis else ""
    if seniority and job.seniority and seniority.lower() == job.seniority.lower():
        evidence.append(f"seniority match: {job.seniority}")

    if job.lifecycle_stage != "active":
        evidence.append(f"lifecycle: {job.lifecycle_stage}")
    if job.presentation.work_mode_label:
        evidence.append(f"job mode: {job.presentation.work_mode_label}")
    elif job.remote_type:
        evidence.append(f"job mode: {job.remote_type}")
    if job.presentation.location_label:
        evidence.append(f"location: {job.presentation.location_label}")
    elif job.primary_variant:
        evidence.append(f"source: {job.primary_variant.source}")

    return evidence[:4]


def score_job(
    profile: EngineProfile, job: EngineJobLifecycle
) -> tuple[int, list[str], list[str], list[str]]:
    profile_values = profile_terms(profile)
    job_values = job_terms(job)
    profile_index = indexed_terms(profile_values)
    job_index = indexed_terms(job_values)

    if not profile_index:
        return 0, [], [], ["profile has no usable terms yet"]

    job_keys = {key for key, _, _ in job_index}
    matched_terms = [display for key, display, _ in profile_index if key in job_keys]
    title_keys = {key for key, _, _ in indexed_terms(tokenize(job.title))}
    title_matches = [display for key, display, _ in profile_index if key in title_keys]

    total_weight = sum(weight for _, _, weight in profile_index)
    matched_weight = sum(weight for key, _, weight in profile_index if key in job_keys)
    title_weight = sum(weight for key, _, weight in profile_index if key in title_keys)

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

    missing_terms = [display for key, display, _ in profile_index if key not in job_keys][:8]
    evidence = build_evidence(profile, job, matched_terms, title_matches)
    return score, matched_terms[:10], missing_terms, evidence
