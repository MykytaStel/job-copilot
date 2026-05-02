from __future__ import annotations

import fcntl
from io import TextIOWrapper
from pathlib import Path


class ProfileBootstrapAlreadyRunningError(Exception):
    pass


class ProfileBootstrapLock:
    def __init__(self, lock_path: Path) -> None:
        self._lock_path = lock_path
        self._handle: TextIOWrapper | None = None

    def acquire(self) -> None:
        self._lock_path.parent.mkdir(parents=True, exist_ok=True)
        handle = self._lock_path.open("a+", encoding="utf-8")
        try:
            fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as exc:
            handle.close()
            raise ProfileBootstrapAlreadyRunningError(
                f"bootstrap already running for lock {self._lock_path.stem}"
            ) from exc
        self._handle = handle

    def release(self) -> None:
        if self._handle is None:
            return
        try:
            fcntl.flock(self._handle.fileno(), fcntl.LOCK_UN)
        finally:
            self._handle.close()
            self._handle = None
