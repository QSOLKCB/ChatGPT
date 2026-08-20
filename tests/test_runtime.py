import unittest

from qsol_chatgpt.model import Action, Approval
from qsol_chatgpt.runtime import Runtime


class FakeTerminal:
    def __init__(self):
        self.calls = []

    def execute(self, argv):
        self.calls.append(list(argv))
        return {"executed": True, "argv": list(argv), "returncode": 0, "stdout": "ok", "stderr": ""}


class RuntimeTests(unittest.TestCase):
    def test_effect_does_not_execute_without_approval(self):
        terminal = FakeTerminal()
        runtime = Runtime(terminal=terminal)
        action = Action(kind="shell.exec", args={"argv": ["printf", "hello"]})
        receipt = runtime.run(action)
        self.assertEqual(receipt.status, "approval_required")
        self.assertEqual(terminal.calls, [])

    def test_mismatched_approval_does_not_execute(self):
        terminal = FakeTerminal()
        runtime = Runtime(terminal=terminal)
        action = Action(kind="shell.exec", args={"argv": ["printf", "hello"]})
        other = Action(kind="shell.exec", args={"argv": ["printf", "other"]})
        approval = Approval(action_id=other.action_id, approved=True, approved_by="tester")
        receipt = runtime.run(action, approval)
        self.assertEqual(receipt.status, "approval_required")
        self.assertEqual(terminal.calls, [])

    def test_exact_approval_executes(self):
        terminal = FakeTerminal()
        runtime = Runtime(terminal=terminal)
        action = Action(kind="shell.exec", args={"argv": ["printf", "hello"]})
        approval = Approval(action_id=action.action_id, approved=True, approved_by="tester")
        receipt = runtime.run(action, approval)
        self.assertEqual(receipt.status, "completed")
        self.assertEqual(terminal.calls, [["printf", "hello"]])

    def test_known_unimplemented_observation_is_receipted(self):
        runtime = Runtime()
        action = Action(kind="screen.capture")
        receipt = runtime.run(action)
        self.assertEqual(receipt.status, "unsupported")
        self.assertEqual(receipt.decision, "allow")


if __name__ == "__main__":
    unittest.main()
