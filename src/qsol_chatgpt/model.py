from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from .receipts import canonical_hash

ACTION_SCHEMA_VERSION = "qsol-chatgpt-action/1"
APPROVAL_SCHEMA_VERSION = "qsol-chatgpt-approval/1"
RECEIPT_SCHEMA_VERSION = "qsol-chatgpt-receipt/1"


@dataclass(frozen=True)
class Action:
    kind: str
    args: dict[str, Any] = field(default_factory=dict)
    requested_by: str = "agent"
    action_id: str = ""
    schema_version: str = ACTION_SCHEMA_VERSION

    def __post_init__(self) -> None:
        if not self.kind:
            raise ValueError("action kind must not be empty")
        if not self.requested_by:
            raise ValueError("requested_by must not be empty")
        expected = canonical_hash(
            {"kind": self.kind, "args": self.args, "requested_by": self.requested_by}
        )
        if self.action_id and self.action_id != expected:
            raise ValueError("action_id does not match canonical action content")
        object.__setattr__(self, "action_id", expected)

    def to_dict(self) -> dict[str, Any]:
        return {
            "schema_version": self.schema_version,
            "action_id": self.action_id,
            "kind": self.kind,
            "args": self.args,
            "requested_by": self.requested_by,
        }


@dataclass(frozen=True)
class Approval:
    action_id: str
    approved: bool
    approved_by: str
    schema_version: str = APPROVAL_SCHEMA_VERSION

    def permits(self, action: Action) -> bool:
        return self.approved and self.action_id == action.action_id

    def to_dict(self) -> dict[str, Any]:
        return {
            "schema_version": self.schema_version,
            "action_id": self.action_id,
            "approved": self.approved,
            "approved_by": self.approved_by,
        }


@dataclass(frozen=True)
class PolicyDecision:
    outcome: str
    reason: str


@dataclass(frozen=True)
class Receipt:
    receipt_id: str
    action_id: str
    kind: str
    decision: str
    status: str
    output: dict[str, Any] | None
    error: str | None
    recorded_at: str
    schema_version: str = RECEIPT_SCHEMA_VERSION

    def to_dict(self) -> dict[str, Any]:
        return {
            "schema_version": self.schema_version,
            "receipt_id": self.receipt_id,
            "action_id": self.action_id,
            "kind": self.kind,
            "decision": self.decision,
            "status": self.status,
            "output": self.output,
            "error": self.error,
            "recorded_at": self.recorded_at,
        }
