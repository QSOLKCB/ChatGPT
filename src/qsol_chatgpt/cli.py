from __future__ import annotations

import argparse
import json

from .model import Action, Approval
from .policy import PolicyEngine
from .runtime import Runtime
from .terminal import TerminalExecutor


def _action_from_json(raw: str) -> Action:
    payload = json.loads(raw)
    return Action(
        kind=payload["kind"],
        args=payload.get("args", {}),
        requested_by=payload.get("requested_by", "agent"),
        action_id=payload.get("action_id", ""),
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="qsol-chatgpt")
    sub = parser.add_subparsers(dest="command", required=True)

    policy = sub.add_parser("policy", help="evaluate an action without execution")
    policy.add_argument("action_json")

    run = sub.add_parser("run", help="evaluate and optionally execute an action")
    run.add_argument("action_json")
    run.add_argument("--approve", action="store_true", help="approve exactly this action")
    run.add_argument("--approved-by", default="local-user")
    run.add_argument("--execute", action="store_true", help="enable the shell executor")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    action = _action_from_json(args.action_json)

    if args.command == "policy":
        decision = PolicyEngine().evaluate(action)
        print(json.dumps({"action": action.to_dict(), "decision": decision.__dict__}, indent=2))
        return 0

    approval = None
    if args.approve:
        approval = Approval(
            action_id=action.action_id,
            approved=True,
            approved_by=args.approved_by,
        )

    runtime = Runtime(terminal=TerminalExecutor(enabled=args.execute))
    receipt = runtime.run(action, approval)
    print(json.dumps(receipt.to_dict(), indent=2))
    return 0 if receipt.status in {"simulated", "completed", "unsupported"} else 2


if __name__ == "__main__":
    raise SystemExit(main())
