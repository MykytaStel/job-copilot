from typing import Any, get_args

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
