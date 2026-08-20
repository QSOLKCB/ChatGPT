from __future__ import annotations

from .model import Action, Approval, Receipt
from .policy import PolicyEngine
from .receipts import build_receipt
from .terminal import TerminalExecutor


class Runtime:
    def __init__(
        self,
        *,
        policy: PolicyEngine | None = None,
        terminal: TerminalExecutor | None = None,
    ) -> None:
        self.policy = policy or PolicyEngine()
        self.terminal = terminal or TerminalExecutor(enabled=False)

    def run(self, action: Action, approval: Approval | None = None) -> Receipt:
        decision = self.policy.evaluate(action)

        if decision.outcome == "deny":
            return build_receipt(
                action=action,
                decision=decision.outcome,
                status="denied",
                error=decision.reason,
            )

        if decision.outcome == "require_approval":
            if approval is None or not approval.permits(action):
                return build_receipt(
                    action=action,
                    decision=decision.outcome,
                    status="approval_required",
                    error="missing or mismatched approval",
                )

        if action.kind == "shell.exec":
            try:
                result = self.terminal.execute(action.args["argv"])
            except Exception as exc:  # executor errors are receipted, not raised across boundary
                return build_receipt(
                    action=action,
                    decision=decision.outcome,
                    status="failed",
                    error=f"{type(exc).__name__}: {exc}",
                )
            status = "completed" if result.get("executed") else "simulated"
            return build_receipt(
                action=action,
                decision=decision.outcome,
                status=status,
                output=result,
            )

        return build_receipt(
            action=action,
            decision=decision.outcome,
            status="unsupported",
            error="no executor registered for capability",
        )
