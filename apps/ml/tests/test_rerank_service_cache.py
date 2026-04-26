import asyncio
from contextlib import asynccontextmanager

from app.api_models import RerankRequest
from app.engine_api_client import EngineJobLifecycle, EngineJobPresentation, EngineProfile
from app.rerank_service import RerankService


def _fake_profile(profile_id: str = "p1") -> EngineProfile:
    return EngineProfile(
        id=profile_id,
        name="Alice",
        email="alice@example.com",
        raw_text="software engineer",
        created_at="2024-01-01T00:00:00Z",
        updated_at="2024-01-01T00:00:00Z",
    )


def _fake_job(job_id: str = "j1") -> EngineJobLifecycle:
    return EngineJobLifecycle(
        id=job_id,
        title="Engineer",
        company_name="Acme",
        description_text="software engineer role",
        first_seen_at="2024-01-01T00:00:00Z",
        last_seen_at="2024-01-01T00:00:00Z",
        is_active=True,
        lifecycle_stage="active",
        presentation=EngineJobPresentation(title="Engineer", company="Acme", badges=[]),
    )


class _FakeEngineClient:
    def __init__(self) -> None:
        self.fetch_count = 0

    async def fetch_profile(self, profile_id: str) -> EngineProfile:
        self.fetch_count += 1
        return _fake_profile(profile_id)

    async def fetch_job_lifecycle(self, job_id: str) -> EngineJobLifecycle:
        self.fetch_count += 1
        return _fake_job(job_id)


def _make_service() -> tuple[RerankService, _FakeEngineClient]:
    client = _FakeEngineClient()

    @asynccontextmanager
    async def factory():
        yield client

    return RerankService(factory), client


def test_rerank_returns_cached_result_on_second_call():
    service, client = _make_service()
    req = RerankRequest(profile_id="p1", job_ids=["j1"])

    result1 = asyncio.run(service.rerank(req))
    calls_after_first = client.fetch_count

    result2 = asyncio.run(service.rerank(req))

    assert result1.jobs == result2.jobs
    assert client.fetch_count == calls_after_first  # no new upstream calls


def test_rerank_cache_bust_skips_cache_and_refetches():
    service, client = _make_service()

    asyncio.run(service.rerank(RerankRequest(profile_id="p1", job_ids=["j1"])))
    calls_after_first = client.fetch_count

    asyncio.run(service.rerank(RerankRequest(profile_id="p1", job_ids=["j1"], cache_bust=True)))

    assert client.fetch_count > calls_after_first


def test_invalidate_clears_profile_cache():
    service, client = _make_service()

    asyncio.run(service.rerank(RerankRequest(profile_id="p1", job_ids=["j1"])))
    calls_after_first = client.fetch_count

    service.invalidate("p1")

    asyncio.run(service.rerank(RerankRequest(profile_id="p1", job_ids=["j1"])))

    assert client.fetch_count > calls_after_first


def test_invalidate_does_not_affect_other_profiles():
    service, client = _make_service()

    asyncio.run(service.rerank(RerankRequest(profile_id="p1", job_ids=["j1"])))
    asyncio.run(service.rerank(RerankRequest(profile_id="p2", job_ids=["j1"])))
    calls_baseline = client.fetch_count

    service.invalidate("p1")

    # p2 cache should still be warm — no new fetches
    asyncio.run(service.rerank(RerankRequest(profile_id="p2", job_ids=["j1"])))
    assert client.fetch_count == calls_baseline


def test_cache_bust_stores_fresh_result_for_next_call():
    service, client = _make_service()

    asyncio.run(service.rerank(RerankRequest(profile_id="p1", job_ids=["j1"])))
    asyncio.run(service.rerank(RerankRequest(profile_id="p1", job_ids=["j1"], cache_bust=True)))
    calls_after_bust = client.fetch_count

    # result from bust call should now be in cache — second normal call is a hit
    asyncio.run(service.rerank(RerankRequest(profile_id="p1", job_ids=["j1"])))
    assert client.fetch_count == calls_after_bust
