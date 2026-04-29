from __future__ import annotations

from app.engine_api_client import EngineJobLifecycle, EngineProfile


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
