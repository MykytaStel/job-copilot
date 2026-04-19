import pytest

from app.llm_provider import TemplateEnrichmentProvider, build_profile_insights_provider
from app.profile_insights import ProfileInsightsProviderError
from app.settings import DEFAULT_CORS_ALLOWED_ORIGINS, get_runtime_settings


@pytest.fixture(autouse=True)
def reset_runtime_settings_cache():
    get_runtime_settings.cache_clear()
    yield
    get_runtime_settings.cache_clear()


def test_runtime_settings_defaults_to_local_dev_cors_origins(monkeypatch):
    monkeypatch.delenv("ML_CORS_ALLOWED_ORIGINS", raising=False)
    monkeypatch.delenv("ML_LOG_LEVEL", raising=False)

    settings = get_runtime_settings()

    assert settings.cors_allowed_origins == DEFAULT_CORS_ALLOWED_ORIGINS
    assert settings.log_level == "INFO"


def test_runtime_settings_reads_explicit_cors_origin_list(monkeypatch):
    monkeypatch.setenv(
        "ML_CORS_ALLOWED_ORIGINS",
        "https://app.example.com, https://ops.example.com",
    )
    monkeypatch.setenv("ML_LOG_LEVEL", "debug")

    settings = get_runtime_settings()

    assert settings.cors_allowed_origins == (
        "https://app.example.com",
        "https://ops.example.com",
    )
    assert settings.log_level == "DEBUG"


def test_build_profile_insights_provider_defaults_to_template(monkeypatch):
    monkeypatch.delenv("ML_LLM_PROVIDER", raising=False)
    monkeypatch.delenv("OPENAI_API_KEY", raising=False)

    provider = build_profile_insights_provider()

    assert isinstance(provider, TemplateEnrichmentProvider)


def test_openai_provider_requires_api_key(monkeypatch):
    monkeypatch.setenv("ML_LLM_PROVIDER", "openai")
    monkeypatch.delenv("OPENAI_API_KEY", raising=False)

    with pytest.raises(ProfileInsightsProviderError, match="OPENAI_API_KEY"):
        build_profile_insights_provider()
