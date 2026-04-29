import os

import pytest
from fastapi.testclient import TestClient

from app.api import app
from app.api_models import BootstrapResponse
from app.bootstrap.locking import ProfileBootstrapAlreadyRunningError, ProfileBootstrapLock
from app.bootstrap.task_store import BootstrapTaskStore


def test_bootstrap_task_store_persists_status_transitions(tmp_path):
    store = BootstrapTaskStore(tmp_path, task_ttl_hours=24)
    created = store.create(task_id="task-1", profile_id="profile-1")
    assert created.status == "accepted"

    running = store.mark_running("task-1")
    assert running.status == "running"
    assert running.started_at is not None

    completed = store.mark_completed(
        "task-1",
        BootstrapResponse(
            retrained=True,
            example_count=30,
            profile_id="profile-1",
            artifact_path="/tmp/profile-1.json",
        ),
    )
    loaded = store.get("task-1")

    assert completed.status == "completed"
    assert loaded is not None
    assert loaded.result is not None
    assert loaded.result.profile_id == "profile-1"
    assert loaded.artifact_path == "/tmp/profile-1.json"
    assert loaded.finished_at is not None


def test_bootstrap_task_store_cleans_expired_files(tmp_path):
    store = BootstrapTaskStore(tmp_path, task_ttl_hours=1)
    store.create(task_id="task-old", profile_id="profile-1")
    task_path = tmp_path / "tasks" / "task-old.json"
    stale_ts = task_path.stat().st_mtime - 7200
    os.utime(task_path, (stale_ts, stale_ts))

    store.cleanup_expired()

    assert store.get("task-old") is None


def test_bootstrap_task_store_reports_status_counts(tmp_path):
    store = BootstrapTaskStore(tmp_path, task_ttl_hours=24)
    store.create(task_id="accepted-task", profile_id="profile-1")
    store.create(task_id="running-task", profile_id="profile-1")
    store.mark_running("running-task")
    store.create(task_id="completed-task", profile_id="profile-1")
    store.mark_completed(
        "completed-task",
        BootstrapResponse(retrained=False, example_count=12, profile_id="profile-1"),
    )
    store.create(task_id="failed-task", profile_id="profile-1")
    store.mark_failed(task_id="failed-task", profile_id="profile-1", error="boom")

    counts = store.status_counts()

    assert counts == {
        "accepted": 1,
        "running": 1,
        "completed": 1,
        "failed": 1,
    }


def test_profile_bootstrap_lock_rejects_second_holder(tmp_path):
    path = tmp_path / "locks" / "profile-1.lock"
    first = ProfileBootstrapLock(path)
    second = ProfileBootstrapLock(path)

    first.acquire()
    try:
        with pytest.raises(ProfileBootstrapAlreadyRunningError):
            second.acquire()
    finally:
        first.release()


class _ReadyResponse:
    def __init__(self, payload):
        self.content = b"{}"
        self._payload = payload

    def json(self):
        return self._payload


def test_ready_route_reports_not_ready_when_engine_api_unavailable():
    async def failing_get(*args, **kwargs):
        raise RuntimeError("boom")

    with TestClient(app) as client:
        client.app.state.services.http_client.get = failing_get
        response = client.get("/ready")

    assert response.status_code == 503
    body = response.json()
    assert body["status"] == "not_ready"
    assert body["components"]["database"]["status"] == "error"
    assert body["components"]["ml_sidecar"]["status"] == "ok"
    assert body["components"]["ingestion"]["status"] == "stale"


def test_ready_route_returns_component_status_from_engine_api():
    async def ready_get(*args, **kwargs):
        return _ReadyResponse(
            {
                "status": "degraded",
                "components": {
                    "database": {"status": "ok", "latency_ms": 4},
                    "ml_sidecar": {"status": "ok"},
                    "ingestion": {"status": "stale", "last_run_at": "2026-04-29 10:00:00+00"},
                },
            }
        )

    with TestClient(app) as client:
        client.app.state.services.http_client.get = ready_get
        response = client.get("/ready")

    body = response.json()
    assert response.status_code == 200
    assert body["status"] == "degraded"
    assert body["components"]["database"] == {"status": "ok", "latency_ms": 4}
    assert body["components"]["ml_sidecar"] == {"status": "ok"}
    assert body["components"]["ingestion"]["status"] == "stale"


def test_metrics_route_exposes_ml_status_metrics():
    with TestClient(app) as client:
        response = client.get("/metrics")

    assert response.status_code == 200
    assert response.headers["content-type"].startswith("text/plain")
    body = response.text
    assert "job_copilot_ml_info" in body
    assert "job_copilot_ml_enrichment_provider_available" in body
    assert 'job_copilot_ml_bootstrap_tasks{status="accepted"}' in body
