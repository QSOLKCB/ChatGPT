import json
import pathlib
import unittest


class SchemaTests(unittest.TestCase):
    def test_schema_files_are_valid_json_and_versioned(self):
        root = pathlib.Path(__file__).resolve().parents[1] / "schemas"
        expected = {"action.schema.json", "approval.schema.json", "receipt.schema.json"}
        self.assertEqual({p.name for p in root.glob("*.json")}, expected)
        for path in root.glob("*.json"):
            data = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(data["$schema"], "https://json-schema.org/draft/2020-12/schema")
            self.assertIn("$id", data)
            self.assertEqual(data["type"], "object")


if __name__ == "__main__":
    unittest.main()
