from __future__ import annotations

from .model import Action, PolicyDecision


class PolicyEngine:
    """Small bootstrap policy kernel. This is a safety floor, not a sandbox."""

    OBSERVE_ONLY = {"screen.capture", "filesystem.read"}
    EFFECTFUL = {
        "shell.exec",
        "input.click",
        "input.type",
        "app.launch",
        "filesystem.write",
    }

    _FORBIDDEN_PROGRAMS = {
        "mkfs",
        "mkfs.ext2",
        "mkfs.ext3",
        "mkfs.ext4",
        "mkfs.xfs",
        "shutdown",
        "reboot",
        "poweroff",
        "halt",
    }

    def evaluate(self, action: Action) -> PolicyDecision:
        if action.kind == "shell.exec":
            denied = self._check_shell(action)
            if denied:
                return PolicyDecision("deny", denied)
            return PolicyDecision("require_approval", "shell execution is effectful")

        if action.kind in self.OBSERVE_ONLY:
            return PolicyDecision("allow", "known bootstrap observation capability")

        if action.kind in self.EFFECTFUL:
            return PolicyDecision("require_approval", "known effectful capability")

        return PolicyDecision("deny", "unknown capability kind")

    def _check_shell(self, action: Action) -> str | None:
        argv = action.args.get("argv")
        if not isinstance(argv, list) or not argv or not all(isinstance(x, str) for x in argv):
            return "shell.exec requires a non-empty string argv array"
        if len(argv) > 256:
            return "shell.exec argv exceeds bootstrap bound"

        program = argv[0].rsplit("/", 1)[-1].lower()
        if program in self._FORBIDDEN_PROGRAMS or program.startswith("mkfs."):
            return f"program is forbidden by bootstrap policy: {program}"

        normalized = " ".join(argv).lower().strip()
        if normalized.startswith("rm -rf /") or normalized.startswith("rm -fr /"):
            return "catastrophic root removal pattern is forbidden"
        if program == "dd" and any(arg.startswith("of=/dev/") for arg in argv[1:]):
            return "raw device writes are forbidden"

        return None
