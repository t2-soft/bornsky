import unittest
import json
from pathlib import Path
import subprocess
import tempfile
from check_public_boundary import content_errors, dependency_errors, history_errors


class BoundaryTests(unittest.TestCase):
    def test_blocks_private_paths(self):
        for path in ("catalog/en.toml", "crates/api/src/lib.rs", ".env.local", "apps/web/page.tsx"):
            self.assertTrue(content_errors(path, "synthetic"))

    def test_reports_secret_without_exposing_it(self):
        fake = "ghp_" + "synthetic" * 5
        errors = content_errors("README.md", fake)
        self.assertTrue(errors)
        self.assertNotIn(fake, " ".join(errors))

    def test_public_scientific_reference_is_allowed(self):
        self.assertEqual(content_errors("docs/accuracy.md", "https://ssd.jpl.nasa.gov/horizons/"), [])

    def test_blocks_local_workspace_and_user_paths(self):
        for parts, separator in [(("C:", "Users", "Example", "file"), chr(92)),
                                 (("C:", "astro", "repo"), "/"),
                                 (("", "home", "example", "file"), "/"),
                                 (("", "Users", "Example", "file"), "/")]:
            self.assertTrue(content_errors("README.md", separator.join(parts)))

    def test_history_finds_deleted_secret_without_echoing_it(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            def git(*args):
                return subprocess.check_output(["git", *args], cwd=root, stderr=subprocess.DEVNULL)
            git("init")
            git("config", "user.name", "Synthetic Test")
            git("config", "user.email", "test@example.invalid")
            (root / "export-manifest.json").write_text(json.dumps({"files": ["README.md", "export-manifest.json"]}), encoding="utf-8")
            fake = "ghp_" + "synthetic" * 5
            (root / "README.md").write_text(fake, encoding="utf-8")
            git("add", ".")
            git("commit", "-m", "Synthetic fixture")
            (root / "README.md").write_text("Clean current contents", encoding="utf-8")
            git("commit", "-am", "Remove fixture")
            errors = history_errors(root)
            self.assertTrue(any("possible credential" in error for error in errors))
            self.assertNotIn(fake, " ".join(errors))

    def test_rejects_dependency_source_and_hidden_build_dependency(self):
        root = {"workspace": {"members": ["crates/astro-engine"], "package": {"publish": False},
                "dependencies": {"serde": "1", "vsop87": "3", "proptest": "1", "serde_json": "1"}}}
        crate = {"dependencies": {k: {"workspace": True} for k in ("serde", "vsop87")},
                 "dev-dependencies": {k: {"workspace": True} for k in ("proptest", "serde_json")}}
        self.assertEqual(dependency_errors(root, crate, {}), [])
        crate["build-dependencies"] = {"unreviewed": "1"}
        self.assertTrue(dependency_errors(root, crate, {}))
        del crate["build-dependencies"]
        root["workspace"]["dependencies"]["serde"] = {"git": "https://example.invalid/private"}
        self.assertTrue(dependency_errors(root, crate, {}))


if __name__ == "__main__":
    unittest.main()
