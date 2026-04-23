import json
import subprocess
import sys
from typing import Any, get_args

from app.application_coach import ApplicationCoachRequest as LegacyApplicationCoachRequest
from app.cover_letter_draft import CoverLetterDraftRequest as LegacyCoverLetterDraftRequest
from app.interview_prep import InterviewPrepRequest as LegacyInterviewPrepRequest
from app.job_fit_explanation import JobFitExplanationRequest as LegacyJobFitExplanationRequest
from app.profile_insights import LlmContextRequest as LegacyLlmContextRequest
from app.weekly_guidance import WeeklyGuidanceRequest as LegacyWeeklyGuidanceRequest
from app.enrichment.application_coach import ApplicationCoachPrompt, ApplicationCoachRequest
from app.enrichment.cover_letter_draft import CoverLetterDraftPrompt, CoverLetterDraftRequest
from app.enrichment.interview_prep import InterviewPrepPrompt, InterviewPrepRequest
from app.enrichment.job_fit_explanation import (
    JobFitExplanationPrompt,
    JobFitExplanationRequest,
)
from app.enrichment.profile_insights import LlmContextRequest, ProfileInsightsPrompt
from app.enrichment.weekly_guidance import WeeklyGuidancePrompt, WeeklyGuidanceRequest
from app.llm_provider_template import TemplateEnrichmentProvider
from app.llm_provider_types import (
    ApplicationCoachProvider,
    CoverLetterDraftProvider,
    InterviewPrepProvider,
    JobFitExplanationProvider,
    ProfileInsightsProvider,
    WeeklyGuidanceProvider,
)
from app.llm_providers.common import PromptPayload
from app.llm_providers.ollama_provider import OllamaEnrichmentProvider
from app.llm_providers.openai_provider import OpenAIEnrichmentProvider


def _loaded_enrichment_modules(module_name: str) -> set[str]:
    script = """
import importlib
import json
import sys

importlib.import_module(sys.argv[1])
loaded = sorted(name for name in sys.modules if name.startswith("app.enrichment"))
print(json.dumps(loaded))
"""
    completed = subprocess.run(
        [sys.executable, "-c", script, module_name],
        check=True,
        capture_output=True,
        text=True,
    )
    return set(json.loads(completed.stdout))


def test_prompt_payload_uses_enrichment_prompt_exports():
    assert get_args(PromptPayload) == (
        ProfileInsightsPrompt,
        JobFitExplanationPrompt,
        ApplicationCoachPrompt,
        CoverLetterDraftPrompt,
        InterviewPrepPrompt,
        WeeklyGuidancePrompt,
    )


def test_provider_protocol_annotations_use_enrichment_request_exports():
    assert ProfileInsightsProvider.generate_profile_insights.__annotations__ == {
        "context": LlmContextRequest,
        "prompt": ProfileInsightsPrompt,
        "return": Any,
    }
    assert JobFitExplanationProvider.generate_job_fit_explanation.__annotations__ == {
        "context": JobFitExplanationRequest,
        "prompt": JobFitExplanationPrompt,
        "return": Any,
    }
    assert ApplicationCoachProvider.generate_application_coach.__annotations__ == {
        "context": ApplicationCoachRequest,
        "prompt": ApplicationCoachPrompt,
        "return": Any,
    }
    assert CoverLetterDraftProvider.generate_cover_letter_draft.__annotations__ == {
        "context": CoverLetterDraftRequest,
        "prompt": CoverLetterDraftPrompt,
        "return": Any,
    }
    assert InterviewPrepProvider.generate_interview_prep.__annotations__ == {
        "context": InterviewPrepRequest,
        "prompt": InterviewPrepPrompt,
        "return": Any,
    }
    assert WeeklyGuidanceProvider.generate_weekly_guidance.__annotations__ == {
        "context": WeeklyGuidanceRequest,
        "prompt": WeeklyGuidancePrompt,
        "return": Any,
    }


def test_provider_implementations_annotate_contexts_from_enrichment_packages():
    assert TemplateEnrichmentProvider.generate_profile_insights.__annotations__["context"] is LlmContextRequest
    assert TemplateEnrichmentProvider.generate_job_fit_explanation.__annotations__["context"] is JobFitExplanationRequest
    assert TemplateEnrichmentProvider.generate_application_coach.__annotations__["context"] is ApplicationCoachRequest
    assert TemplateEnrichmentProvider.generate_cover_letter_draft.__annotations__["context"] is CoverLetterDraftRequest
    assert TemplateEnrichmentProvider.generate_interview_prep.__annotations__["context"] is InterviewPrepRequest
    assert TemplateEnrichmentProvider.generate_weekly_guidance.__annotations__["context"] is WeeklyGuidanceRequest

    assert OpenAIEnrichmentProvider.generate_profile_insights.__annotations__["context"] is LlmContextRequest
    assert OpenAIEnrichmentProvider.generate_job_fit_explanation.__annotations__["context"] is JobFitExplanationRequest
    assert OpenAIEnrichmentProvider.generate_application_coach.__annotations__["context"] is ApplicationCoachRequest
    assert OpenAIEnrichmentProvider.generate_cover_letter_draft.__annotations__["context"] is CoverLetterDraftRequest
    assert OpenAIEnrichmentProvider.generate_interview_prep.__annotations__["context"] is InterviewPrepRequest
    assert OpenAIEnrichmentProvider.generate_weekly_guidance.__annotations__["context"] is WeeklyGuidanceRequest

    assert OllamaEnrichmentProvider.generate_profile_insights.__annotations__["context"] is LlmContextRequest
    assert OllamaEnrichmentProvider.generate_job_fit_explanation.__annotations__["context"] is JobFitExplanationRequest
    assert OllamaEnrichmentProvider.generate_application_coach.__annotations__["context"] is ApplicationCoachRequest
    assert OllamaEnrichmentProvider.generate_cover_letter_draft.__annotations__["context"] is CoverLetterDraftRequest
    assert OllamaEnrichmentProvider.generate_interview_prep.__annotations__["context"] is InterviewPrepRequest
    assert OllamaEnrichmentProvider.generate_weekly_guidance.__annotations__["context"] is WeeklyGuidanceRequest


def test_legacy_flat_modules_remain_aliases_of_enrichment_exports():
    assert LegacyLlmContextRequest is LlmContextRequest
    assert LegacyJobFitExplanationRequest is JobFitExplanationRequest
    assert LegacyApplicationCoachRequest is ApplicationCoachRequest
    assert LegacyCoverLetterDraftRequest is CoverLetterDraftRequest
    assert LegacyInterviewPrepRequest is InterviewPrepRequest
    assert LegacyWeeklyGuidanceRequest is WeeklyGuidanceRequest


def test_prompt_and_parser_imports_do_not_pull_unrelated_enrichment_reexports():
    expected_absent = {
        "app.enrichment.application_coach.prompt": {
            "app.enrichment.job_fit_explanation.prompt",
            "app.enrichment.job_fit_explanation.parser",
            "app.enrichment.profile_insights.prompt",
            "app.enrichment.profile_insights.parser",
        },
        "app.enrichment.application_coach.parser": {
            "app.enrichment.job_fit_explanation.prompt",
            "app.enrichment.job_fit_explanation.parser",
            "app.enrichment.profile_insights.prompt",
            "app.enrichment.profile_insights.parser",
        },
        "app.enrichment.cover_letter_draft.prompt": {
            "app.enrichment.application_coach.prompt",
            "app.enrichment.application_coach.parser",
            "app.enrichment.job_fit_explanation.prompt",
            "app.enrichment.job_fit_explanation.parser",
            "app.enrichment.profile_insights.prompt",
            "app.enrichment.profile_insights.parser",
        },
        "app.enrichment.cover_letter_draft.parser": {
            "app.enrichment.application_coach.prompt",
            "app.enrichment.application_coach.parser",
            "app.enrichment.job_fit_explanation.prompt",
            "app.enrichment.job_fit_explanation.parser",
            "app.enrichment.profile_insights.prompt",
            "app.enrichment.profile_insights.parser",
        },
        "app.enrichment.interview_prep.prompt": {
            "app.enrichment.application_coach.prompt",
            "app.enrichment.application_coach.parser",
            "app.enrichment.cover_letter_draft.prompt",
            "app.enrichment.cover_letter_draft.parser",
            "app.enrichment.job_fit_explanation.prompt",
            "app.enrichment.job_fit_explanation.parser",
            "app.enrichment.profile_insights.prompt",
            "app.enrichment.profile_insights.parser",
        },
        "app.enrichment.interview_prep.parser": {
            "app.enrichment.application_coach.prompt",
            "app.enrichment.application_coach.parser",
            "app.enrichment.cover_letter_draft.prompt",
            "app.enrichment.cover_letter_draft.parser",
            "app.enrichment.job_fit_explanation.prompt",
            "app.enrichment.job_fit_explanation.parser",
            "app.enrichment.profile_insights.prompt",
            "app.enrichment.profile_insights.parser",
        },
        "app.enrichment.job_fit_explanation.prompt": {
            "app.enrichment.profile_insights.prompt",
            "app.enrichment.profile_insights.parser",
        },
        "app.enrichment.job_fit_explanation.parser": {
            "app.enrichment.profile_insights.prompt",
            "app.enrichment.profile_insights.parser",
        },
        "app.enrichment.weekly_guidance.prompt": {
            "app.enrichment.profile_insights.prompt",
            "app.enrichment.profile_insights.parser",
        },
        "app.enrichment.weekly_guidance.parser": {
            "app.enrichment.profile_insights.prompt",
            "app.enrichment.profile_insights.parser",
        },
    }

    for module_name, absent_modules in expected_absent.items():
        loaded_modules = _loaded_enrichment_modules(module_name)
        assert absent_modules.isdisjoint(loaded_modules), module_name
