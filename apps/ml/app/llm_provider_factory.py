from app.enrichment.application_coach.contract import ApplicationCoachProviderError
from app.enrichment.cover_letter_draft.contract import CoverLetterDraftProviderError
from app.enrichment.interview_prep.contract import InterviewPrepProviderError
from app.enrichment.job_fit_explanation.contract import JobFitExplanationProviderError
from app.enrichment.profile_insights.contract import ProfileInsightsProviderError
from app.enrichment.weekly_guidance.errors import WeeklyGuidanceProviderError
from app.llm_provider_remote import OllamaEnrichmentProvider, OpenAIEnrichmentProvider
from app.llm_provider_template import TemplateEnrichmentProvider
from app.settings import get_runtime_settings


EnrichmentProvider = (
    TemplateEnrichmentProvider | OpenAIEnrichmentProvider | OllamaEnrichmentProvider
)


def build_enrichment_provider() -> EnrichmentProvider:
    settings = get_runtime_settings()
    provider_name = settings.llm_provider

    if provider_name == "template":
        return TemplateEnrichmentProvider()

    if provider_name == "openai":
        api_key = settings.openai_api_key or ""
        if not api_key:
            raise ProfileInsightsProviderError("OPENAI_API_KEY is required when ML_LLM_PROVIDER=openai.")

        return OpenAIEnrichmentProvider(
            api_key=api_key,
            model=settings.openai_model,
            base_url=settings.openai_base_url,
        )

    if provider_name == "ollama":
        return OllamaEnrichmentProvider(
            base_url=settings.ollama_base_url,
            model=settings.ollama_model,
            timeout_seconds=settings.llm_request_timeout_seconds,
        )

    raise ProfileInsightsProviderError(f"Unsupported ML_LLM_PROVIDER: {provider_name}")


def build_profile_insights_provider() -> EnrichmentProvider:
    provider = build_enrichment_provider()
    return provider


def build_job_fit_explanation_provider() -> EnrichmentProvider:
    try:
        provider = build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise JobFitExplanationProviderError(str(exc)) from exc
    return provider


def build_application_coach_provider() -> EnrichmentProvider:
    try:
        provider = build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise ApplicationCoachProviderError(str(exc)) from exc
    return provider


def build_cover_letter_draft_provider() -> EnrichmentProvider:
    try:
        provider = build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise CoverLetterDraftProviderError(str(exc)) from exc
    return provider


def build_interview_prep_provider() -> EnrichmentProvider:
    try:
        provider = build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise InterviewPrepProviderError(str(exc)) from exc
    return provider


def build_weekly_guidance_provider() -> EnrichmentProvider:
    try:
        provider = build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise WeeklyGuidanceProviderError(str(exc)) from exc
    return provider
