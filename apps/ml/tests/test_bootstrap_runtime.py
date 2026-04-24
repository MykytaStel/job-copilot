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


def test_ready_route_reports_degraded_engine_api(monkeypatch):
    async def failing_probe(self):
        raise RuntimeError("boom")

    monkeypatch.setattr("app.api.EngineApiClient.probe_health", failing_probe)

    with TestClient(app) as client:
        response = client.get("/ready")

    assert response.status_code == 200
    body = response.json()
    assert body["status"] == "degraded"
    checks = {item["name"]: item for item in body["checks"]}
    assert checks["engine_api"]["status"] == "degraded"


def test_ready_route_reports_bootstrap_runtime_saturation():
    with TestClient(app) as client:
        services = client.app.state.services
        services.task_store.create(task_id="accepted-1", profile_id="profile-1")
        services.task_store.create(task_id="accepted-2", profile_id="profile-2")
        services.reranker_bootstrap_service._active_jobs = (
            services.reranker_bootstrap_service._max_concurrent_jobs
        )

        response = client.get("/ready")

    body = response.json()
    checks = {item["name"]: item for item in body["checks"]}
    assert checks["bootstrap_runtime"]["status"] == "degraded"
    assert "active=" in (checks["bootstrap_runtime"]["detail"] or "")
    assert "queued=2" in (checks["bootstrap_runtime"]["detail"] or "")
    assert "accepted=2" in (checks["bootstrap_runtime"]["detail"] or "")
