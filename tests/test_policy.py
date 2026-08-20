import unittest

from qsol_chatgpt.model import Action
from qsol_chatgpt.policy import PolicyEngine


class PolicyTests(unittest.TestCase):
    def setUp(self):
        self.policy = PolicyEngine()

    def test_unknown_capability_is_denied(self):
        decision = self.policy.evaluate(Action(kind="teleport.house"))
        self.assertEqual(decision.outcome, "deny")

    def test_observation_capability_is_allowed(self):
        decision = self.policy.evaluate(Action(kind="screen.capture"))
        self.assertEqual(decision.outcome, "allow")

    def test_shell_requires_approval(self):
        action = Action(kind="shell.exec", args={"argv": ["printf", "hello"]})
        decision = self.policy.evaluate(action)
        self.assertEqual(decision.outcome, "require_approval")

    def test_malformed_shell_action_is_denied(self):
        action = Action(kind="shell.exec", args={"command": "printf hello"})
        decision = self.policy.evaluate(action)
        self.assertEqual(decision.outcome, "deny")

    def test_catastrophic_root_removal_is_denied(self):
        action = Action(kind="shell.exec", args={"argv": ["rm", "-rf", "/"]})
        decision = self.policy.evaluate(action)
        self.assertEqual(decision.outcome, "deny")

    def test_raw_device_write_is_denied(self):
        action = Action(kind="shell.exec", args={"argv": ["dd", "if=/tmp/x", "of=/dev/sda"]})
        decision = self.policy.evaluate(action)
        self.assertEqual(decision.outcome, "deny")


if __name__ == "__main__":
    unittest.main()
