import asyncio
import re

import httpx
from fastapi import FastAPI, HTTPException, status
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field

from app.engine_api_client import (
    EngineApiClient,
    EngineJobLifecycle,
    EngineProfile,
    engine_api_base_url,
    engine_api_timeout_seconds,
)


TOKEN_RE = re.compile(r"[a-z0-9]+")
STOPWORDS = {
    "a",
    "an",
    "and",
    "are",
    "as",
    "at",
    "be",
    "by",
    "for",
    "from",
    "in",
    "into",
    "is",
    "of",
    "on",
    "or",
    "the",
    "to",
    "with",
}

app = FastAPI(
    title="job-copilot-ml",
    version="0.1.0",
    description="Read-only ML sidecar over canonical engine-api data.",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)


class HealthResponse(BaseModel):
    status: str
    service: str
    engine_api_base_url: str


class FitAnalyzeRequest(BaseModel):
    profile_id: str = Field(min_length=1)
    job_id: str = Field(min_length=1)


class FitAnalyzeResponse(BaseModel):
    profile_id: str
    job_id: str
    score: int
    matched_terms: list[str]
    missing_terms: list[str]
    evidence: list[str]


class RerankRequest(BaseModel):
    profile_id: str = Field(min_length=1)
    job_ids: list[str] = Field(min_length=1, max_length=50)


class RerankedJob(BaseModel):
    job_id: str
    title: str
    company_name: str
    score: int
    matched_terms: list[str]
    evidence: list[str]


class RerankResponse(BaseModel):
    profile_id: str
    jobs: list[RerankedJob]


def normalize_text(value: str) -> str:
    return (
        value.lower()
        .replace("c++", "cpp")
        .replace("c#", "csharp")
        .replace("node.js", "nodejs")
        .replace("react.js", "react")
        .replace("react native", "reactnative")
    )


def tokenize(*chunks: str | None) -> list[str]:
    tokens: list[str] = []
    for chunk in chunks:
        if not chunk:
            continue
        normalized = normalize_text(chunk)
        for token in TOKEN_RE.findall(normalized):
            if len(token) < 2 or token in STOPWORDS:
                continue
            tokens.append(token)
    return tokens


def unique_preserving_order(values: list[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        result.append(value)
    return result


def profile_terms(profile: EngineProfile) -> list[str]:
    analysis = profile.analysis
    if analysis:
        terms = tokenize(
            analysis.primary_role,
            analysis.seniority,
            " ".join(analysis.skills),
            " ".join(analysis.keywords),
            profile.raw_text,
        )
    else:
        terms = tokenize(profile.raw_text)
    return unique_preserving_order(terms)[:40]


def job_terms(job: EngineJobLifecycle) -> list[str]:
    return unique_preserving_order(
        tokenize(
            job.title,
            job.company_name,
            job.remote_type,
            job.seniority,
            job.description_text,
        )
    )


def overlap(profile_values: list[str], job_values: list[str]) -> list[str]:
    job_set = set(job_values)
    return [value for value in profile_values if value in job_set]


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
    if job.remote_type:
        evidence.append(f"job mode: {job.remote_type}")
    elif job.primary_variant:
        evidence.append(f"source: {job.primary_variant.source}")

    return evidence[:4]


def score_job(
    profile: EngineProfile, job: EngineJobLifecycle
) -> tuple[int, list[str], list[str], list[str]]:
    profile_values = profile_terms(profile)
    job_values = job_terms(job)

    if not profile_values:
        return 0, [], [], ["profile has no usable terms yet"]

    matched_terms = overlap(profile_values, job_values)
    title_values = set(tokenize(job.title))
    title_matches = [value for value in matched_terms if value in title_values]

    overlap_ratio = len(matched_terms) / len(profile_values)
    title_bonus = min(len(title_matches) * 8, 24)

    seniority_bonus = 0
    if profile.analysis and profile.analysis.seniority and job.seniority:
        if profile.analysis.seniority.lower() == job.seniority.lower():
            seniority_bonus = 10

    active_bonus = 5 if job.is_active else 0
    lifecycle_bonus = 3 if job.lifecycle_stage == "reactivated" else 0
    score = min(
        int(round(overlap_ratio * 70)) + title_bonus + seniority_bonus + active_bonus + lifecycle_bonus,
        100,
    )

    missing_terms = [value for value in profile_values if value not in set(job_values)][:8]
    evidence = build_evidence(profile, job, matched_terms, title_matches)
    return score, matched_terms[:10], missing_terms, evidence


@app.get("/health", response_model=HealthResponse)
async def health() -> HealthResponse:
    return HealthResponse(
        status="ok",
        service="ml",
        engine_api_base_url=engine_api_base_url(),
    )


@app.post("/api/v1/fit/analyze", response_model=FitAnalyzeResponse)
async def analyze_fit(payload: FitAnalyzeRequest) -> FitAnalyzeResponse:
    timeout = httpx.Timeout(engine_api_timeout_seconds())
    async with httpx.AsyncClient(timeout=timeout) as client:
        engine_api = EngineApiClient(client)
        profile = await engine_api.fetch_profile(payload.profile_id)
        job = await engine_api.fetch_job_lifecycle(payload.job_id)

    score, matched_terms, missing_terms, evidence = score_job(profile, job)
    return FitAnalyzeResponse(
        profile_id=payload.profile_id,
        job_id=payload.job_id,
        score=score,
        matched_terms=matched_terms,
        missing_terms=missing_terms,
        evidence=evidence,
    )


@app.post("/api/v1/rerank", response_model=RerankResponse)
async def rerank_jobs(payload: RerankRequest) -> RerankResponse:
    unique_job_ids = unique_preserving_order([job_id.strip() for job_id in payload.job_ids if job_id.strip()])
    if not unique_job_ids:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="job_ids must contain at least one non-empty id",
        )

    timeout = httpx.Timeout(engine_api_timeout_seconds())
    async with httpx.AsyncClient(timeout=timeout) as client:
        engine_api = EngineApiClient(client)
        profile = await engine_api.fetch_profile(payload.profile_id)
        jobs = await asyncio.gather(
            *(engine_api.fetch_job_lifecycle(job_id) for job_id in unique_job_ids)
        )

    ranked_jobs: list[RerankedJob] = []
    for job in jobs:
        score, matched_terms, _, evidence = score_job(profile, job)
        ranked_jobs.append(
            RerankedJob(
                job_id=job.id,
                title=job.title,
                company_name=job.company_name,
                score=score,
                matched_terms=matched_terms,
                evidence=evidence,
            )
        )

    ranked_jobs.sort(key=lambda item: (-item.score, item.title.lower(), item.job_id))
    return RerankResponse(profile_id=payload.profile_id, jobs=ranked_jobs)
