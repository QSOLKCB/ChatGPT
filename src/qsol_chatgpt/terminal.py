from __future__ import annotations

import subprocess
from typing import Any


class TerminalExecutor:
    """Structured argv executor. Disabled unless explicitly enabled by caller."""

    def __init__(self, *, enabled: bool = False, timeout_seconds: float = 30.0) -> None:
        self.enabled = enabled
        self.timeout_seconds = timeout_seconds

    def execute(self, argv: list[str]) -> dict[str, Any]:
        if not self.enabled:
            return {
                "executed": False,
                "reason": "execution_disabled",
                "argv": list(argv),
            }

        completed = subprocess.run(
            argv,
            shell=False,
            capture_output=True,
            text=True,
            timeout=self.timeout_seconds,
            check=False,
            env={},
        )
        return {
            "executed": True,
            "argv": list(argv),
            "returncode": completed.returncode,
            "stdout": completed.stdout,
            "stderr": completed.stderr,
        }
