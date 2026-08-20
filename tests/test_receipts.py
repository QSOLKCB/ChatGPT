import unittest

from qsol_chatgpt.model import Action
from qsol_chatgpt.receipts import build_receipt


class ReceiptTests(unittest.TestCase):
    def test_action_identity_is_deterministic(self):
        a = Action(kind="filesystem.read", args={"path": "/tmp/example"})
        b = Action(kind="filesystem.read", args={"path": "/tmp/example"})
        self.assertEqual(a.action_id, b.action_id)

    def test_action_identity_changes_with_arguments(self):
        a = Action(kind="filesystem.read", args={"path": "/tmp/a"})
        b = Action(kind="filesystem.read", args={"path": "/tmp/b"})
        self.assertNotEqual(a.action_id, b.action_id)

    def test_receipt_identity_excludes_wall_clock_time(self):
        action = Action(kind="screen.capture")
        a = build_receipt(action=action, decision="allow", status="unsupported", error="no executor")
        b = build_receipt(action=action, decision="allow", status="unsupported", error="no executor")
        self.assertEqual(a.receipt_id, b.receipt_id)


if __name__ == "__main__":
    unittest.main()
