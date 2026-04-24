import pytest

from app.llm_provider import TemplateEnrichmentProvider, build_profile_insights_provider
from app.llm_provider_factory import build_enrichment_provider
from app.profile_insights import ProfileInsightsProviderError
from app.settings import (
    DEFAULT_BOOTSTRAP_MAX_CONCURRENT_JOBS,
    DEFAULT_CORS_ALLOWED_ORIGINS,
    DEFAULT_ENGINE_API_BASE_URL,
    DEFAULT_ENGINE_API_TIMEOUT_SECONDS,
    DEFAULT_LLM_PROVIDER,
    DEFAULT_LLM_REQUEST_TIMEOUT_SECONDS,
    DEFAULT_OLLAMA_BASE_URL,
    DEFAULT_OLLAMA_MODEL,
    DEFAULT_OPENAI_MODEL,
    get_runtime_settings,
)


@pytest.fixture(autouse=True)
def reset_runtime_settings_cache():
    get_runtime_settings.cache_clear()
    yield
    get_runtime_settings.cache_clear()


def test_runtime_settings_defaults_to_local_dev_cors_origins(monkeypatch):
    monkeypatch.delenv("ML_CORS_ALLOWED_ORIGINS", raising=False)
    monkeypatch.delenv("ML_LOG_LEVEL", raising=False)
    monkeypatch.delenv("ENGINE_API_BASE_URL", raising=False)
    monkeypatch.delenv("ENGINE_API_TIMEOUT_SECONDS", raising=False)
    monkeypatch.delenv("ML_LLM_PROVIDER", raising=False)
    monkeypatch.delenv("OPENAI_MODEL", raising=False)
    monkeypatch.delenv("OLLAMA_BASE_URL", raising=False)
    monkeypatch.delenv("OLLAMA_MODEL", raising=False)

    settings = get_runtime_settings()

    assert settings.cors_allowed_origins == DEFAULT_CORS_ALLOWED_ORIGINS
    assert settings.log_level == "INFO"
    assert settings.engine_api_base_url == DEFAULT_ENGINE_API_BASE_URL
    assert settings.engine_api_timeout_seconds == DEFAULT_ENGINE_API_TIMEOUT_SECONDS
    assert settings.llm_provider == DEFAULT_LLM_PROVIDER
    assert settings.openai_model == DEFAULT_OPENAI_MODEL
    assert settings.ollama_base_url == DEFAULT_OLLAMA_BASE_URL
    assert settings.ollama_model == DEFAULT_OLLAMA_MODEL
    assert settings.llm_request_timeout_seconds == DEFAULT_LLM_REQUEST_TIMEOUT_SECONDS
    assert settings.bootstrap_max_concurrent_jobs == DEFAULT_BOOTSTRAP_MAX_CONCURRENT_JOBS


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


def test_runtime_settings_read_llm_timeout_and_bootstrap_concurrency(monkeypatch):
    monkeypatch.setenv("ML_LLM_REQUEST_TIMEOUT_SECONDS", "12.5")
    monkeypatch.setenv("ML_BOOTSTRAP_MAX_CONCURRENT_JOBS", "4")

    settings = get_runtime_settings()

    assert settings.llm_request_timeout_seconds == 12.5
    assert settings.bootstrap_max_concurrent_jobs == 4


def test_build_enrichment_provider_passes_timeout_to_ollama(monkeypatch):
    captured: dict[str, object] = {}

    class FakeOllamaProvider:
        def __init__(self, *, base_url: str, model: str, timeout_seconds: float):
            captured["base_url"] = base_url
            captured["model"] = model
            captured["timeout_seconds"] = timeout_seconds

    monkeypatch.setenv("ML_LLM_PROVIDER", "ollama")
    monkeypatch.setenv("OLLAMA_BASE_URL", "http://ollama.test")
    monkeypatch.setenv("OLLAMA_MODEL", "llama-test")
    monkeypatch.setenv("ML_LLM_REQUEST_TIMEOUT_SECONDS", "22.5")
    monkeypatch.setattr("app.llm_provider_factory.OllamaEnrichmentProvider", FakeOllamaProvider)

    provider = build_enrichment_provider()

    assert isinstance(provider, FakeOllamaProvider)
    assert captured == {
        "base_url": "http://ollama.test",
        "model": "llama-test",
        "timeout_seconds": 22.5,
    }


def test_build_enrichment_provider_passes_timeout_to_openai(monkeypatch):
    captured: dict[str, object] = {}

    class FakeOpenAIProvider:
        def __init__(
            self,
            *,
            api_key: str,
            model: str,
            base_url: str | None = None,
            timeout_seconds: float | None = None,
        ):
            captured["api_key"] = api_key
            captured["model"] = model
            captured["base_url"] = base_url
            captured["timeout_seconds"] = timeout_seconds

    monkeypatch.setenv("ML_LLM_PROVIDER", "openai")
    monkeypatch.setenv("OPENAI_API_KEY", "test-key")
    monkeypatch.setenv("OPENAI_MODEL", "gpt-test")
    monkeypatch.setenv("OPENAI_BASE_URL", "https://openai.test")
    monkeypatch.setenv("ML_LLM_REQUEST_TIMEOUT_SECONDS", "18.0")
    monkeypatch.setattr("app.llm_provider_factory.OpenAIEnrichmentProvider", FakeOpenAIProvider)

    provider = build_enrichment_provider()

    assert isinstance(provider, FakeOpenAIProvider)
    assert captured == {
        "api_key": "test-key",
        "model": "gpt-test",
        "base_url": "https://openai.test",
        "timeout_seconds": 18.0,
    }
