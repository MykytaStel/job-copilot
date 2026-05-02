import asyncio
import sys
import types

import httpx
import pytest

from app.llm_provider_remote import OllamaEnrichmentProvider, OpenAIEnrichmentProvider
from app.profile_insights import (
    LlmContextRequest,
    ProfileInsightsPrompt,
    ProfileInsightsProviderError,
)


def sample_context() -> LlmContextRequest:
    return LlmContextRequest.model_validate(
        {
            "profile_id": "profile-1",
            "jobs_feed_summary": {
                "total": 10,
                "active": 8,
                "inactive": 1,
                "reactivated": 1,
            },
            "feedback_summary": {
                "saved_jobs_count": 1,
                "hidden_jobs_count": 0,
                "bad_fit_jobs_count": 0,
                "whitelisted_companies_count": 0,
                "blacklisted_companies_count": 0,
            },
        }
    )


def sample_prompt() -> ProfileInsightsPrompt:
    return ProfileInsightsPrompt(
        system_instructions="Return JSON only.",
        context_payload='{"ok": true}',
        output_schema_expectations='{"type":"object"}',
        output_schema={"type": "object"},
    )


def test_openai_provider_retries_transient_errors_and_returns_output(monkeypatch):
    calls = {"count": 0}

    class RetryableError(Exception):
        pass

    class FakeOpenAI:
        def __init__(
            self,
            api_key: str,
            base_url: str | None = None,
            timeout: float | None = None,
        ):
            self.responses = types.SimpleNamespace(create=self.create)

        def create(self, **kwargs):
            calls["count"] += 1
            if calls["count"] == 1:
                raise RetryableError("temporary")
            return types.SimpleNamespace(output_text='{"profile_summary":"ok"}')

    fake_module = types.ModuleType("openai")
    fake_module.OpenAI = FakeOpenAI
    fake_module.APIConnectionError = RetryableError
    fake_module.APITimeoutError = RetryableError
    fake_module.InternalServerError = RetryableError
    fake_module.RateLimitError = RetryableError
    monkeypatch.setitem(sys.modules, "openai", fake_module)

    provider = OpenAIEnrichmentProvider(api_key="key", model="gpt-test")
    result = asyncio.run(provider.generate_profile_insights(sample_context(), sample_prompt()))

    assert result == '{"profile_summary":"ok"}'
    assert calls["count"] == 2


def test_openai_provider_raises_on_empty_response(monkeypatch):
    class RetryableError(Exception):
        pass

    class FakeOpenAI:
        def __init__(
            self,
            api_key: str,
            base_url: str | None = None,
            timeout: float | None = None,
        ):
            self.responses = types.SimpleNamespace(create=self.create)

        def create(self, **kwargs):
            return types.SimpleNamespace(output_text="")

    fake_module = types.ModuleType("openai")
    fake_module.OpenAI = FakeOpenAI
    fake_module.APIConnectionError = RetryableError
    fake_module.APITimeoutError = RetryableError
    fake_module.InternalServerError = RetryableError
    fake_module.RateLimitError = RetryableError
    monkeypatch.setitem(sys.modules, "openai", fake_module)

    provider = OpenAIEnrichmentProvider(api_key="key", model="gpt-test")

    with pytest.raises(ProfileInsightsProviderError, match="empty response"):
        asyncio.run(provider.generate_profile_insights(sample_context(), sample_prompt()))


def test_openai_provider_passes_timeout_to_sdk(monkeypatch):
    captured: dict[str, object] = {}

    class RetryableError(Exception):
        pass

    class FakeOpenAI:
        def __init__(
            self,
            api_key: str,
            base_url: str | None = None,
            timeout: float | None = None,
        ):
            captured["api_key"] = api_key
            captured["base_url"] = base_url
            captured["timeout"] = timeout
            self.responses = types.SimpleNamespace(create=self.create)

        def create(self, **kwargs):
            return types.SimpleNamespace(output_text='{"profile_summary":"ok"}')

    fake_module = types.ModuleType("openai")
    fake_module.OpenAI = FakeOpenAI
    fake_module.APIConnectionError = RetryableError
    fake_module.APITimeoutError = RetryableError
    fake_module.InternalServerError = RetryableError
    fake_module.RateLimitError = RetryableError
    monkeypatch.setitem(sys.modules, "openai", fake_module)

    provider = OpenAIEnrichmentProvider(
        api_key="key",
        model="gpt-test",
        base_url="https://openai.test",
        timeout_seconds=12.5,
    )
    result = asyncio.run(provider.generate_profile_insights(sample_context(), sample_prompt()))

    assert result == '{"profile_summary":"ok"}'
    assert captured == {
        "api_key": "key",
        "base_url": "https://openai.test",
        "timeout": 12.5,
    }


def test_ollama_provider_retries_transient_errors_and_returns_output(monkeypatch):
    calls = {"count": 0}

    class FakeResponse:
        def raise_for_status(self) -> None:
            return None

        def json(self) -> dict:
            return {"message": {"content": '{"profile_summary":"ok"}'}}

    class FakeAsyncClient:
        async def post(self, url: str, json: dict):
            calls["count"] += 1
            if calls["count"] == 1:
                raise httpx.ConnectError("temporary")
            return FakeResponse()

    monkeypatch.setattr(
        "app.llm_providers.ollama_provider.build_async_client",
        lambda timeout_seconds: FakeAsyncClient(),
    )

    provider = OllamaEnrichmentProvider(base_url="http://ollama.test", model="llama")
    result = asyncio.run(provider.generate_profile_insights(sample_context(), sample_prompt()))

    assert result == '{"profile_summary":"ok"}'
    assert calls["count"] == 2


def test_ollama_provider_does_not_retry_http_status_errors(monkeypatch):
    calls = {"count": 0}
    request = httpx.Request("POST", "http://ollama.test/api/chat")
    response = httpx.Response(status_code=500, request=request)

    class FakeAsyncClient:
        async def post(self, url: str, json: dict):
            calls["count"] += 1
            raise httpx.HTTPStatusError("boom", request=request, response=response)

    monkeypatch.setattr(
        "app.llm_providers.ollama_provider.build_async_client",
        lambda timeout_seconds: FakeAsyncClient(),
    )

    provider = OllamaEnrichmentProvider(base_url="http://ollama.test", model="llama")

    with pytest.raises(ProfileInsightsProviderError, match="Ollama request failed"):
        asyncio.run(provider.generate_profile_insights(sample_context(), sample_prompt()))

    assert calls["count"] == 1
