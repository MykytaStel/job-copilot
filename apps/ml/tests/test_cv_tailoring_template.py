from app.cv_tailoring import (
    CvTailoringRequest,
    MalformedCvTailoringOutputError,
    parse_cv_tailoring_suggestions,
)
from app.template_cv_tailoring import build_cv_tailoring


def sample_request() -> CvTailoringRequest:
    return CvTailoringRequest.model_validate(
        {
            "profile_id": "profile-1",
            "job_id": "job-1",
            "profile_summary": "Senior backend engineer with Rust and Postgres experience.",
            "candidate_skills": ["Rust", "Postgres", "Docker", "Python"],
            "job_title": "Senior Rust Backend Engineer",
            "job_description": (
                "Build and maintain distributed backend systems using Rust and Postgres."
            ),
            "job_required_skills": ["Rust", "Postgres", "Distributed Systems"],
            "job_nice_to_have_skills": ["Docker", "Kafka"],
            "candidate_cv_text": (
                "Led development of Rust microservices for a fintech platform."
            ),
        }
    )


def empty_request() -> CvTailoringRequest:
    return CvTailoringRequest.model_validate(
        {
            "profile_id": "profile-empty",
            "job_id": "job-empty",
        }
    )


def test_template_highlights_overlapping_required_skills() -> None:
    result = build_cv_tailoring(sample_request())

    assert result["skills_to_highlight"] == ["Rust", "Postgres"]


def test_template_mentions_overlapping_nice_to_have_skills() -> None:
    result = build_cv_tailoring(sample_request())

    assert result["skills_to_mention"] == ["Docker"]


def test_template_reports_missing_required_skills_as_gaps() -> None:
    result = build_cv_tailoring(sample_request())

    gap_skills = [gap["skill"] for gap in result["gaps_to_address"]]

    assert "Distributed Systems" in gap_skills


def test_template_gap_suggestions_are_actionable() -> None:
    result = build_cv_tailoring(sample_request())

    distributed_systems_gap = next(
        gap for gap in result["gaps_to_address"] if gap["skill"] == "Distributed Systems"
    )

    assert distributed_systems_gap["suggestion"]
    assert "Distributed Systems" in distributed_systems_gap["suggestion"]


def test_template_summary_rewrite_is_non_empty_and_mentions_job_title() -> None:
    result = build_cv_tailoring(sample_request())

    assert result["summary_rewrite"]
    assert "Senior Rust Backend Engineer" in result["summary_rewrite"]


def test_template_empty_input_returns_safe_defaults() -> None:
    result = build_cv_tailoring(empty_request())

    assert result["skills_to_highlight"] == []
    assert result["skills_to_mention"] == []
    assert result["gaps_to_address"] == []
    assert result["summary_rewrite"]
    assert result["key_phrases"] == []


def test_template_key_phrases_include_required_and_nice_to_have_skills() -> None:
    result = build_cv_tailoring(sample_request())

    assert result["key_phrases"] == [
        "Rust",
        "Postgres",
        "Distributed Systems",
        "Docker",
        "Kafka",
    ]


def test_parser_normalizes_and_deduplicates_cv_tailoring_output() -> None:
    parsed = parse_cv_tailoring_suggestions(
        {
            "skills_to_highlight": ["- Rust", " rust ", "Postgres"],
            "skills_to_mention": ["1. Docker"],
            "gaps_to_address": [
                {
                    "skill": "Kafka",
                    "suggestion": "Add Kafka examples.",
                }
            ],
            "summary_rewrite": "```json Senior Rust engineer.```",
            "key_phrases": ["Rust", "rust"],
        }
    )

    assert parsed.skills_to_highlight == ["Rust", "Postgres"]
    assert parsed.skills_to_mention == ["Docker"]
    assert parsed.gaps_to_address[0].skill == "Kafka"
    assert parsed.summary_rewrite == "Senior Rust engineer."
    assert parsed.key_phrases == ["Rust"]


def test_parser_rejects_non_json_string_output() -> None:
    try:
        parse_cv_tailoring_suggestions("not json {{{")
    except MalformedCvTailoringOutputError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected MalformedCvTailoringOutputError")


def test_parser_rejects_invalid_gap_shape() -> None:
    try:
        parse_cv_tailoring_suggestions(
            {
                "skills_to_highlight": [],
                "skills_to_mention": [],
                "gaps_to_address": [
                    {
                        "skill": 123,
                        "suggestion": "bad",
                    }
                ],
                "summary_rewrite": "",
                "key_phrases": [],
            }
        )
    except MalformedCvTailoringOutputError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected MalformedCvTailoringOutputError")
