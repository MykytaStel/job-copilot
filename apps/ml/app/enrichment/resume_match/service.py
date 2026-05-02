from __future__ import annotations

import math
from collections import Counter
from dataclasses import dataclass

from app.enrichment.resume_match.contract import (
    MAX_GAP_SUMMARY_LENGTH,
    MAX_KEYWORD_LENGTH,
    MAX_KEYWORDS,
    ResumeMatchRequest,
    ResumeMatchResponse,
)
from app.text_normalization import LOW_SIGNAL_TERMS, STOPWORDS, normalize_text

MIN_KEYWORD_LENGTH = 2
TOP_JD_KEYWORDS = 24


@dataclass(frozen=True)
class WeightedKeyword:
    term: str
    weight: float


class ResumeMatchService:
    def analyze(self, payload: ResumeMatchRequest) -> ResumeMatchResponse:
        resume_terms = _extract_weighted_terms(payload.resume_text)
        jd_terms = _extract_weighted_terms(payload.jd_text)
        jd_keywords = _rank_jd_keywords(jd_terms, resume_terms)

        if not jd_keywords:
            return ResumeMatchResponse(
                keyword_coverage_percent=0.0,
                matched_keywords=[],
                missing_keywords=[],
                gap_summary="The job description does not contain enough specific keywords to compare against the resume text.",
            )

        resume_term_set = set(resume_terms)
        matched = [keyword for keyword in jd_keywords if keyword.term in resume_term_set]
        missing = [keyword for keyword in jd_keywords if keyword.term not in resume_term_set]
        total_weight = sum(keyword.weight for keyword in jd_keywords)
        matched_weight = sum(keyword.weight for keyword in matched)
        coverage = round((matched_weight / total_weight) * 100, 1) if total_weight > 0 else 0.0

        matched_keywords = [_format_keyword(keyword.term) for keyword in matched[:MAX_KEYWORDS]]
        missing_keywords = [_format_keyword(keyword.term) for keyword in missing[:MAX_KEYWORDS]]

        return ResumeMatchResponse(
            keyword_coverage_percent=coverage,
            matched_keywords=matched_keywords,
            missing_keywords=missing_keywords,
            gap_summary=_build_gap_summary(coverage, missing_keywords),
        )


def _extract_weighted_terms(text: str) -> Counter[str]:
    normalized = normalize_text(text)
    if not normalized:
        return Counter()

    tokens = normalized.split()
    counts: Counter[str] = Counter()

    for token in tokens:
        if _is_keyword(token):
            counts[token] += 1

    for left, right in zip(tokens, tokens[1:]):
        if _is_keyword(left) and _is_keyword(right):
            counts[f"{left} {right}"] += 1.4

    return counts


def _rank_jd_keywords(jd_terms: Counter[str], resume_terms: Counter[str]) -> list[WeightedKeyword]:
    ranked: list[WeightedKeyword] = []
    for term, frequency in jd_terms.items():
        doc_frequency = 1 + (1 if term in resume_terms else 0)
        inverse_doc_frequency = math.log((2 + 1) / (doc_frequency + 1)) + 1
        phrase_boost = 1.25 if " " in term or "_" in term else 1.0
        weight = float(frequency) * inverse_doc_frequency * phrase_boost
        ranked.append(WeightedKeyword(term=term, weight=weight))

    ranked.sort(key=lambda keyword: (-keyword.weight, keyword.term))
    return ranked[:TOP_JD_KEYWORDS]


def _is_keyword(term: str) -> bool:
    display_term = term.replace("_", " ")
    if len(display_term) < MIN_KEYWORD_LENGTH:
        return False
    if len(display_term) > MAX_KEYWORD_LENGTH:
        return False
    if term in STOPWORDS or term in LOW_SIGNAL_TERMS:
        return False
    return any(character.isalpha() for character in display_term)


def _format_keyword(term: str) -> str:
    return term.replace("_", " ")


def _build_gap_summary(coverage: float, missing_keywords: list[str]) -> str:
    if not missing_keywords:
        summary = "The resume text covers the strongest extracted keywords from this job description."
    else:
        lead = ", ".join(missing_keywords[:5])
        if coverage >= 75:
            summary = f"Strong keyword overlap. The main remaining gaps are {lead}."
        elif coverage >= 45:
            summary = f"Moderate keyword overlap. Consider adding truthful evidence for {lead}."
        else:
            summary = f"Low keyword overlap. The resume should address core JD terms such as {lead} before applying."

    return summary[:MAX_GAP_SUMMARY_LENGTH]

