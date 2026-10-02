"""Fail closed on unreviewed files/dependencies; never print credential matches."""

from pathlib import Path, PurePosixPath
import json
import re
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
RUNTIME_DEPS = {"serde", "vsop87"}
DEV_DEPS = {"proptest", "serde_json"}
MODULES = {"angle", "aspects", "bodies", "chart", "coords", "ephemeris", "houses", "nutation", "time"}
PRIVATE_PARTS = {"catalog", "catalogs", "api", "apps", "migrations", "backups", "deploy", "node_modules"}
SECRET_PATTERNS = [
    re.compile(r"(?:ghp_|github_pat_|sb_secret_|bsk_live_|sk_live_)[A-Za-z0-9_]{12,}"),
    re.compile(r"AKIA[A-Z0-9]{16}"),
    re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
    re.compile(r"(?:postgres(?:ql)?|mysql)://[^\s/:]+:[^\s@]+@"),
    re.compile(r"eyJ[A-Za-z0-9_-]{16,}\.[A-Za-z0-9_-]{16,}\.[A-Za-z0-9_-]{16,}"),
]
PRIVATE_SOURCE = re.compile(r"bornsky\.ai|skyborn-api\.fly\.dev|supabase\.co|neon\.tech|(?:C:|/Users/|/home/)[/\\]Users", re.I)


def content_errors(name: str, text: str) -> list[str]:
    errors = []
    parts = PurePosixPath(name).parts
    if any(p.lower() in PRIVATE_PARTS or p.lower().startswith(".env") for p in parts):
        errors.append(f"excluded path: {name}")
    if any(pattern.search(text) for pattern in SECRET_PATTERNS):
        errors.append(f"possible credential in {name} (value redacted)")
    if PRIVATE_SOURCE.search(text):
        errors.append(f"private repository/deployment reference in {name}")
    return errors


def dependency_errors(root_manifest: dict, crate_manifest: dict, lock: dict) -> list[str]:
    errors = []
    workspace = root_manifest.get("workspace", {})
    if workspace.get("members") != ["crates/astro-engine"]:
        errors.append("workspace must contain only astro-engine")
    if set(workspace.get("dependencies", {})) != RUNTIME_DEPS | DEV_DEPS:
        errors.append("unreviewed workspace dependency")
    if set(crate_manifest.get("dependencies", {})) != RUNTIME_DEPS:
        errors.append("unreviewed runtime dependency")
    if set(crate_manifest.get("dev-dependencies", {})) != DEV_DEPS:
        errors.append("unreviewed development dependency")
    if any(k in crate_manifest for k in ("build-dependencies", "target", "patch", "replace")):
        errors.append("unreviewed conditional/build dependency configuration")
    if any(k in root_manifest for k in ("patch", "replace", "dependencies", "build-dependencies", "target")):
        errors.append("unreviewed root dependency override")
    if workspace.get("package", {}).get("publish") is not False:
        errors.append("crate publishing must remain disabled until separately reviewed")
    for config in workspace.get("dependencies", {}).values():
        if isinstance(config, dict) and any(k in config for k in ("git", "path", "registry")):
            errors.append("unreviewed workspace dependency source")
    for key in ("dependencies", "dev-dependencies"):
        for config in crate_manifest.get(key, {}).values():
            if config != {"workspace": True}:
                errors.append("crate dependencies must use the approved workspace definitions")
    for package in lock.get("package", []):
        source = package.get("source")
        if source not in (None, "registry+https://github.com/rust-lang/crates.io-index"):
            errors.append("unreviewed lockfile source")
        if source is None and package["name"] != "astro-engine":
            errors.append("unreviewed local package in lockfile")
    return errors


def check(root: Path) -> list[str]:
    manifest = json.loads((root / "export-manifest.json").read_text(encoding="utf-8"))
    approved = manifest["files"]
    if len(set(approved)) != len(approved):
        return ["duplicate export manifest entry"]
    output = subprocess.check_output(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], cwd=root
    )
    files = set(output.decode("utf-8").strip("\0").split("\0")) - {""}
    errors = [f"unreviewed file: {p}" for p in sorted(files - set(approved))]
    errors += [f"missing approved file: {p}" for p in sorted(set(approved) - files)]
    for name in sorted(files):
        path = root / name
        if path.is_symlink() or not path.resolve().is_relative_to(root.resolve()):
            errors.append(f"symlink or external path: {name}")
            continue
        if not path.is_file():
            errors.append(f"missing or non-regular file: {name}")
            continue
        data = path.read_bytes()
        if len(data) > 300_000:
            errors.append(f"unexpected large file: {name}")
            continue
        try:
            text = data.decode("utf-8")
        except UnicodeDecodeError:
            errors.append(f"binary file is not allowed: {name}")
            continue
        errors += content_errors(name, text)
    for manifest_name in ("Cargo.toml", "crates/astro-engine/Cargo.toml", "Cargo.lock"):
        if not (root / manifest_name).is_file():
            return errors + [f"missing Rust manifest: {manifest_name}"]
    configs = [tomllib.loads((root / p).read_text(encoding="utf-8")) for p in
               ("Cargo.toml", "crates/astro-engine/Cargo.toml", "Cargo.lock")]
    errors += dependency_errors(*configs)
    lib = (root / "crates/astro-engine/src/lib.rs").read_text(encoding="utf-8")
    if set(re.findall(r"^(?:pub )?mod (\w+);", lib, re.M)) != MODULES:
        errors.append("unreviewed root Rust module")
    return errors


if __name__ == "__main__":
    problems = check(ROOT)
    for problem in problems:
        print(problem, file=sys.stderr)
    if problems:
        sys.exit(1)
    print("Public boundary passed: approved files, modules, dependency sources and credential-pattern checks.")
