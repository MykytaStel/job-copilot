import importlib.util
from pathlib import Path
import sys


def load_verifier_module():
    script_path = (
        Path(__file__).resolve().parents[3] / "scripts" / "verify_mixed_load_runtime.py"
    )
    spec = importlib.util.spec_from_file_location("verify_mixed_load_runtime", script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_summarize_request_results_reports_latency_and_status_counts():
    verifier = load_verifier_module()

    summary = verifier.summarize_request_results(
        [
            {"ok": True, "status": 200, "duration_ms": 25.0, "returned_jobs": 3},
            {"ok": True, "status": 200, "duration_ms": 35.0, "returned_jobs": 3},
            {"ok": False, "status": 503, "duration_ms": 90.0},
        ]
    )

    assert summary == {
        "requests": 3,
        "errors": 1,
        "success_rate": 0.6667,
        "mean_ms": 50.0,
        "p50_ms": 35.0,
        "p95_ms": 90.0,
        "max_ms": 90.0,
        "status_counts": {"200": 2, "503": 1},
        "avg_returned_jobs": 3.0,
    }


def test_extract_bootstrap_runtime_detail_reads_ready_payload():
    verifier = load_verifier_module()

    ready_status, detail = verifier.extract_bootstrap_runtime_detail(
        {
            "status": "degraded",
            "checks": [
                {"name": "engine_api", "status": "ok"},
                {
                    "name": "bootstrap_runtime",
                    "status": "degraded",
                    "detail": "active=2/2, available_slots=0, queued=3, accepted=1, running=2, completed=5, failed=0",
                },
            ],
        }
    )

    assert ready_status == "degraded"
    assert detail == "active=2/2, available_slots=0, queued=3, accepted=1, running=2, completed=5, failed=0"


def test_local_runtime_cleanup_paths_sanitizes_profile_segments(tmp_path):
    verifier = load_verifier_module()

    paths = verifier.local_runtime_cleanup_paths(
        repo_root=tmp_path,
        profile_ids=[" profile/1 "],
        task_ids=["task-1"],
    )

    assert paths["task_paths"] == [
        tmp_path / "apps" / "ml" / ".runtime" / "bootstrap-tasks" / "tasks" / "task-1.json"
    ]
    assert paths["profile_dirs"] == [
        tmp_path / "apps" / "ml" / "models" / "profiles" / "profile_1"
    ]
    assert paths["runtime_artifact_path"] == (
        tmp_path / "apps" / "ml" / "models" / "trained-reranker-v3.json"
    )


def test_summarize_runtime_samples_extracts_queue_and_degraded_counts():
    verifier = load_verifier_module()

    summary = verifier.summarize_runtime_samples(
        [
            {
                "status": "ok",
                "normalized_runtime": {
                    "active_jobs": 1,
                    "max_concurrent_jobs": 2,
                    "available_slots": 1,
                    "queued": 0,
                    "accepted": 0,
                    "running": 1,
                    "completed": 0,
                    "failed": 0,
                },
            },
            {
                "status": "degraded",
                "normalized_runtime": {
                    "active_jobs": 2,
                    "max_concurrent_jobs": 2,
                    "available_slots": 0,
                    "queued": 3,
                    "accepted": 1,
                    "running": 2,
                    "completed": 1,
                    "failed": 0,
                },
            },
        ]
    )

    assert summary == {
        "sample_count": 2,
        "degraded_samples": 1,
        "max_active_jobs": 2,
        "max_queued_jobs": 3,
        "max_accepted_tasks": 1,
        "max_running_tasks": 2,
        "max_completed_tasks": 1,
        "max_failed_tasks": 0,
    }


def test_cleanup_local_runtime_outputs_removes_generated_runtime_files(tmp_path):
    verifier = load_verifier_module()
    repo_root = tmp_path
    task_path = (
        repo_root / "apps" / "ml" / ".runtime" / "bootstrap-tasks" / "tasks" / "task-1.json"
    )
    old_task_path = (
        repo_root / "apps" / "ml" / ".runtime" / "bootstrap-tasks" / "tasks" / "old-task.json"
    )
    profile_dir = repo_root / "apps" / "ml" / "models" / "profiles" / "profile-1"
    other_profile_dir = repo_root / "apps" / "ml" / "models" / "profiles" / "profile-2"
    runtime_artifact = repo_root / "apps" / "ml" / "models" / "trained-reranker-v3.json"

    task_path.parent.mkdir(parents=True, exist_ok=True)
    task_path.write_text("{}", encoding="utf-8")
    old_task_path.write_text("{}", encoding="utf-8")
    profile_dir.mkdir(parents=True, exist_ok=True)
    (profile_dir / "trained-reranker-v3.json").write_text("{}", encoding="utf-8")
    other_profile_dir.mkdir(parents=True, exist_ok=True)
    (other_profile_dir / "trained-reranker-v3.json").write_text("{}", encoding="utf-8")
    runtime_artifact.parent.mkdir(parents=True, exist_ok=True)
    runtime_artifact.write_text("{}", encoding="utf-8")

    summary = verifier.cleanup_local_runtime_outputs(
        repo_root=repo_root,
        profile_ids=["profile-1"],
        task_ids=["task-1"],
    )

    assert summary == {
        "deleted_task_files": 2,
        "deleted_profile_dirs": 2,
        "runtime_artifact_deleted": True,
        "errors": [],
    }
    assert not task_path.exists()
    assert not old_task_path.exists()
    assert not profile_dir.exists()
    assert not other_profile_dir.exists()
    assert not runtime_artifact.exists()


def test_normalize_runtime_sample_subtracts_baseline_task_counts():
    verifier = load_verifier_module()

    normalized = verifier.normalize_runtime_sample(
        {
            "active_jobs": 2,
            "max_concurrent_jobs": 2,
            "available_slots": 0,
            "queued": 3,
            "accepted": 1,
            "running": 2,
            "completed": 12,
            "failed": 1,
        },
        {
            "active_jobs": 0,
            "max_concurrent_jobs": 2,
            "available_slots": 2,
            "queued": 0,
            "accepted": 0,
            "running": 0,
            "completed": 6,
            "failed": 1,
        },
    )

    assert normalized == {
        "active_jobs": 2,
        "max_concurrent_jobs": 2,
        "available_slots": 0,
        "queued": 3,
        "accepted": 1,
        "running": 2,
        "completed": 6,
        "failed": 0,
    }


def test_parse_docker_stats_payload_extracts_cpu_and_memory_usage():
    verifier = load_verifier_module()

    parsed = verifier.parse_docker_stats_payload(
        {
            "Name": "job-copilot-ml",
            "CPUPerc": "125.5%",
            "MemPerc": "1.25%",
            "MemUsage": "256.0MiB / 15.53GiB",
        }
    )

    assert parsed == {
        "container": "job-copilot-ml",
        "cpu_percent": 125.5,
        "memory_percent": 1.25,
        "memory_usage_mib": 256.0,
        "memory_limit_mib": 15902.72,
    }


def test_summarize_resource_samples_groups_metrics_per_container():
    verifier = load_verifier_module()

    summary = verifier.summarize_resource_samples(
        [
            {
                "container": "job-copilot-ml",
                "cpu_percent": 110.0,
                "memory_percent": 1.2,
                "memory_usage_mib": 240.0,
            },
            {
                "container": "job-copilot-ml",
                "cpu_percent": 150.0,
                "memory_percent": 1.8,
                "memory_usage_mib": 300.0,
            },
            {
                "container": "job-copilot-engine-api",
                "error": "container stats missing",
            },
        ]
    )

    assert summary == {
        "job-copilot-ml": {
            "sample_count": 2,
            "successful_samples": 2,
            "failed_samples": 0,
            "cpu_percent": {"mean": 130.0, "p95": 150.0, "max": 150.0},
            "memory_percent": {"mean": 1.5, "p95": 1.8, "max": 1.8},
            "memory_usage_mib": {"mean": 270.0, "p95": 300.0, "max": 300.0},
        },
        "job-copilot-engine-api": {
            "sample_count": 1,
            "successful_samples": 0,
            "failed_samples": 1,
            "error": "container stats missing",
        },
    }


def test_cleanup_docker_runtime_outputs_skips_when_container_not_configured():
    verifier = load_verifier_module()

    summary = verifier.cleanup_docker_runtime_outputs(
        container_name="",
        profile_ids=["profile-1"],
        task_ids=["task-1"],
        timeout_seconds=1.0,
    )

    assert summary == {
        "container": None,
        "deleted_task_files": 0,
        "deleted_profile_dirs": 0,
        "runtime_artifact_deleted": False,
        "errors": [],
    }


def test_slowdown_ratio_handles_zero_baseline():
    verifier = load_verifier_module()

    assert verifier.slowdown_ratio(0.0, 125.0) == 0.0
    assert verifier.slowdown_ratio(25.0, 100.0) == 4.0


def test_summarize_bootstrap_results_aggregates_terminal_outcomes():
    verifier = load_verifier_module()

    summary = verifier.summarize_bootstrap_results(
        [
            {
                "task_id": "task-1",
                "status": "completed",
                "retrained": True,
                "promotion_decision": "promoted",
                "reason": "temporal spread warning",
                "duration_ms": 500.0,
                "queue_delay_ms": 100.0,
            },
            {
                "task_id": "task-2",
                "status": "completed",
                "retrained": False,
                "promotion_decision": "skipped_min_examples",
                "reason": "need at least 15 examples, got 3",
                "duration_ms": 200.0,
                "queue_delay_ms": 25.0,
            },
        ]
    )

    assert summary == {
        "task_count": 2,
        "terminal_status_counts": {"completed": 2},
        "retrained_count": 1,
        "promotion_decisions": {"promoted": 1, "skipped_min_examples": 1},
        "reasons": {
            "temporal spread warning": 1,
            "need at least 15 examples, got 3": 1,
        },
        "duration_ms": {
            "mean": 350.0,
            "p50": 200.0,
            "p95": 500.0,
            "max": 500.0,
        },
        "queue_delay_ms": {
            "mean": 62.5,
            "p50": 25.0,
            "p95": 100.0,
            "max": 100.0,
        },
    }
