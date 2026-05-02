from __future__ import annotations

import json
import logging
import os
from datetime import UTC, datetime, timedelta
from pathlib import Path

from app.api_models import BootstrapResponse, BootstrapTaskStatus

logger = logging.getLogger(__name__)


def utc_now_iso() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


class BootstrapTaskStore:
    def __init__(self, root_dir: Path, *, task_ttl_hours: int) -> None:
        self._root_dir = root_dir
        self._task_ttl = timedelta(hours=max(1, task_ttl_hours))
        self._tasks_dir = root_dir / "tasks"
        self._locks_dir = root_dir / "locks"
        self._tasks_dir.mkdir(parents=True, exist_ok=True)
        self._locks_dir.mkdir(parents=True, exist_ok=True)

    @property
    def locks_dir(self) -> Path:
        return self._locks_dir

    def create(self, *, task_id: str, profile_id: str) -> BootstrapTaskStatus:
        self.cleanup_expired()
        status = BootstrapTaskStatus(task_id=task_id, profile_id=profile_id, status="accepted")
        self._write(status)
        return status

    def mark_running(self, task_id: str) -> BootstrapTaskStatus:
        status = self.get(task_id)
        if status is None:
            raise KeyError(f"task not found: {task_id}")
        updated = status.model_copy(update={"status": "running", "started_at": utc_now_iso()})
        self._write(updated)
        return updated

    def mark_completed(self, task_id: str, result: BootstrapResponse) -> BootstrapTaskStatus:
        finished_at = result.finished_at or utc_now_iso()
        updated_result = result.model_copy(update={"finished_at": finished_at})
        status = BootstrapTaskStatus(
            task_id=task_id,
            profile_id=updated_result.profile_id,
            status="completed",
            result=updated_result,
            artifact_path=updated_result.artifact_path,
            started_at=updated_result.started_at,
            finished_at=updated_result.finished_at,
            promotion_decision=updated_result.promotion_decision,
            metrics_version=updated_result.metrics_version,
        )
        self._write(status)
        return status

    def mark_failed(
        self,
        task_id: str,
        *,
        profile_id: str,
        error: str,
        artifact_path: str | None = None,
        started_at: str | None = None,
        promotion_decision: str | None = None,
        metrics_version: str | None = None,
    ) -> BootstrapTaskStatus:
        status = BootstrapTaskStatus(
            task_id=task_id,
            profile_id=profile_id,
            status="failed",
            error=error,
            artifact_path=artifact_path,
            started_at=started_at,
            finished_at=utc_now_iso(),
            promotion_decision=promotion_decision,
            metrics_version=metrics_version,
        )
        self._write(status)
        return status

    def get(self, task_id: str) -> BootstrapTaskStatus | None:
        self.cleanup_expired()
        path = self._task_path(task_id)
        if not path.exists():
            return None
        with path.open("r", encoding="utf-8") as handle:
            payload = json.load(handle)
        return BootstrapTaskStatus.model_validate(payload)

    def cleanup_expired(self) -> None:
        cutoff = datetime.now(UTC) - self._task_ttl
        for path in self._tasks_dir.glob("*.json"):
            try:
                modified = datetime.fromtimestamp(path.stat().st_mtime, tz=UTC)
            except FileNotFoundError:
                continue
            if modified >= cutoff:
                continue
            try:
                path.unlink()
            except FileNotFoundError:
                continue

    def status_counts(self) -> dict[str, int]:
        self.cleanup_expired()
        counts = {
            "accepted": 0,
            "running": 0,
            "completed": 0,
            "failed": 0,
        }
        for path in self._tasks_dir.glob("*.json"):
            try:
                with path.open("r", encoding="utf-8") as handle:
                    payload = json.load(handle)
            except FileNotFoundError:
                continue
            except json.JSONDecodeError as exc:
                logger.error(
                    "task file corrupt; skipping",
                    extra={"path": str(path), "error": str(exc)},
                )
                continue
            status = str(payload.get("status", "")).strip().lower()
            if status in counts:
                counts[status] += 1
        return counts

    def _task_path(self, task_id: str) -> Path:
        return self._tasks_dir / f"{task_id}.json"

    def _write(self, status: BootstrapTaskStatus) -> None:
        path = self._task_path(status.task_id)
        path.parent.mkdir(parents=True, exist_ok=True)
        tmp_path = path.with_suffix(".json.tmp")
        payload = status.model_dump_json(indent=2) + "\n"
        with tmp_path.open("w", encoding="utf-8") as handle:
            handle.write(payload)
        os.replace(tmp_path, path)
