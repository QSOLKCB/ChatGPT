from __future__ import annotations

import hashlib
import json
from datetime import datetime, timezone
from typing import Any


def canonical_json(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def canonical_hash(value: Any) -> str:
    return hashlib.sha256(canonical_json(value).encode("utf-8")).hexdigest()


def build_receipt(*, action, decision: str, status: str, output=None, error=None):
    from .model import RECEIPT_SCHEMA_VERSION, Receipt

    identity_payload = {
        "schema_version": RECEIPT_SCHEMA_VERSION,
        "action_id": action.action_id,
        "kind": action.kind,
        "decision": decision,
        "status": status,
        "output": output,
        "error": error,
    }
    return Receipt(
        receipt_id=canonical_hash(identity_payload),
        action_id=action.action_id,
        kind=action.kind,
        decision=decision,
        status=status,
        output=output,
        error=error,
        recorded_at=datetime.now(timezone.utc).isoformat(),
    )
