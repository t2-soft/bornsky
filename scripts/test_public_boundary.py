import unittest
from check_public_boundary import content_errors, dependency_errors


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
