from app.application_coach import ApplicationCoachProviderError
from app.cover_letter_draft import CoverLetterDraftProviderError
from app.interview_prep import InterviewPrepProviderError
from app.job_fit_explanation import JobFitExplanationProviderError
from app.profile_insights import ProfileInsightsProviderError
from app.weekly_guidance import WeeklyGuidanceProviderError

from app.llm_provider_remote import OllamaEnrichmentProvider, OpenAIEnrichmentProvider
from app.llm_provider_template import TemplateEnrichmentProvider
from app.settings import DEFAULT_LLM_REQUEST_TIMEOUT_SECONDS, get_runtime_settings


EnrichmentProvider = (
    TemplateEnrichmentProvider | OpenAIEnrichmentProvider | OllamaEnrichmentProvider
)


def _build_enrichment_provider() -> EnrichmentProvider:
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
            timeout_seconds=DEFAULT_LLM_REQUEST_TIMEOUT_SECONDS,
        )

    raise ProfileInsightsProviderError(f"Unsupported ML_LLM_PROVIDER: {provider_name}")


def build_profile_insights_provider() -> EnrichmentProvider:
    provider = _build_enrichment_provider()
    return provider


def build_job_fit_explanation_provider() -> EnrichmentProvider:
    try:
        provider = _build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise JobFitExplanationProviderError(str(exc)) from exc
    return provider


def build_application_coach_provider() -> EnrichmentProvider:
    try:
        provider = _build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise ApplicationCoachProviderError(str(exc)) from exc
    return provider


def build_cover_letter_draft_provider() -> EnrichmentProvider:
    try:
        provider = _build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise CoverLetterDraftProviderError(str(exc)) from exc
    return provider


def build_interview_prep_provider() -> EnrichmentProvider:
    try:
        provider = _build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise InterviewPrepProviderError(str(exc)) from exc
    return provider


def build_weekly_guidance_provider() -> EnrichmentProvider:
    try:
        provider = _build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise WeeklyGuidanceProviderError(str(exc)) from exc
    return provider
